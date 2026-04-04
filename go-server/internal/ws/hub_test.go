package ws

import (
	"os"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
)

func TestMain(m *testing.M) {
	logger.Init()
	code := m.Run()
	logger.Sync()
	os.Exit(code)
}

// mockClient creates a Client with a full send channel for testing slow clients
func mockClient(hub *Hub) *Client {
	c := &Client{
		hub:  hub,
		send: make(chan []byte, 1), // buffer size 1 - will be full
		conn: nil,                  // not needed for broadcast test
	}
	// Fill the channel to simulate a slow client
	c.send <- []byte("blocking message")
	return c
}

// TestHub_ConcurrentBroadcast tests for race conditions during concurrent broadcasts
// with slow clients that fail to receive messages.
// This test should FAIL with race detector before the fix (RLock→Lock upgrade bug)
func TestHub_ConcurrentBroadcast(t *testing.T) {
	hub := NewHub()
	go hub.Run()
	defer func() {
		// Cleanup: close remaining clients
		hub.mu.Lock()
		for c := range hub.clients {
			close(c.send)
			delete(hub.clients, c)
		}
		hub.mu.Unlock()
	}()

	// Create multiple mock clients with full channels (slow clients)
	numClients := 20
	clients := make([]*Client, numClients)
	for i := 0; i < numClients; i++ {
		clients[i] = mockClient(hub)
		hub.Register(clients[i])
	}

	// Wait for all clients to be registered
	time.Sleep(50 * time.Millisecond)

	// Concurrently broadcast messages to trigger the race condition
	// The race occurs when multiple goroutines try to upgrade RLock→Lock
	// while iterating over the clients map
	numBroadcasts := 100
	var wg sync.WaitGroup
	var broadcastCount int64

	for i := 0; i < numBroadcasts; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			atomic.AddInt64(&broadcastCount, 1)
			hub.BroadcastEvent(WSEvent{
				Event:   "TEST_EVENT",
				Payload: map[string]int{"index": idx},
			})
		}(i)
	}

	wg.Wait()

	// Verify all broadcasts completed
	if atomic.LoadInt64(&broadcastCount) != int64(numBroadcasts) {
		t.Errorf("Expected %d broadcasts, got %d", numBroadcasts, broadcastCount)
	}

	// Give time for hub to process
	time.Sleep(100 * time.Millisecond)

	// Some slow clients should have been disconnected
	// (their channels were full, so they hit the default case)
	hub.mu.RLock()
	remaining := len(hub.clients)
	hub.mu.RUnlock()

	t.Logf("Remaining clients after broadcasts: %d (started with %d)", remaining, numClients)
}

// TestHub_ConcurrentBroadcastWithRegisterUnregister tests race conditions
// when concurrent register/unregister happens during broadcast
func TestHub_ConcurrentBroadcastWithRegisterUnregister(t *testing.T) {
	hub := NewHub()
	go hub.Run()
	defer func() {
		hub.mu.Lock()
		for c := range hub.clients {
			close(c.send)
			delete(hub.clients, c)
		}
		hub.mu.Unlock()
	}()

	var wg sync.WaitGroup
	stopCh := make(chan struct{})

	// Goroutine 1: Continuously register clients
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			select {
			case <-stopCh:
				return
			default:
				c := mockClient(hub)
				hub.Register(c)
				time.Sleep(time.Microsecond)
			}
		}
	}()

	// Goroutine 2: Continuously broadcast
	wg.Add(1)
	go func() {
		defer wg.Done()
		for i := 0; i < 50; i++ {
			select {
			case <-stopCh:
				return
			default:
				hub.BroadcastEvent(WSEvent{
					Event:   "TEST",
					Payload: map[string]int{"i": i},
				})
				time.Sleep(time.Microsecond)
			}
		}
	}()

	// Goroutine 3: Continuously unregister random clients
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			select {
			case <-stopCh:
				return
			default:
				var clientToUnregister *Client
				hub.mu.RLock()
				for c := range hub.clients {
					clientToUnregister = c
					break
				}
				hub.mu.RUnlock()
				if clientToUnregister != nil {
					hub.Unregister(clientToUnregister)
				}
				time.Sleep(time.Microsecond)
			}
		}
	}()

	// Let the race happen
	time.Sleep(200 * time.Millisecond)
	close(stopCh)
	wg.Wait()
}

// BenchmarkHub_Broadcast_100Clients benchmarks broadcast performance with 100 clients
func BenchmarkHub_Broadcast_100Clients(b *testing.B) {
	hub := NewHub()
	go hub.Run()
	defer func() {
		hub.mu.Lock()
		for c := range hub.clients {
			close(c.send)
			delete(hub.clients, c)
		}
		hub.mu.Unlock()
	}()

	// Create 100 clients with buffered channels (fast clients that won't block)
	numClients := 100
	for i := 0; i < numClients; i++ {
		c := &Client{
			hub:  hub,
			send: make(chan []byte, 256), // larger buffer to avoid blocking
			conn: nil,
		}
		hub.Register(c)
	}

	// Wait for registration
	time.Sleep(50 * time.Millisecond)

	event := WSEvent{
		Event:   "BENCH_EVENT",
		Payload: map[string]string{"test": "data"},
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		hub.BroadcastEvent(event)
	}
	b.StopTimer()
}

// BenchmarkHub_Broadcast_WithSlowClients benchmarks with some slow clients
func BenchmarkHub_Broadcast_WithSlowClients(b *testing.B) {
	hub := NewHub()
	go hub.Run()
	defer func() {
		hub.mu.Lock()
		for c := range hub.clients {
			close(c.send)
			delete(hub.clients, c)
		}
		hub.mu.Unlock()
	}()

	// Create 90 fast clients
	for i := 0; i < 90; i++ {
		c := &Client{
			hub:  hub,
			send: make(chan []byte, 256),
			conn: nil,
		}
		hub.Register(c)
	}

	// Create 10 slow clients with full buffers
	for i := 0; i < 10; i++ {
		c := &Client{
			hub:  hub,
			send: make(chan []byte, 1),
			conn: nil,
		}
		c.send <- []byte("blocking")
		hub.Register(c)
	}

	time.Sleep(50 * time.Millisecond)

	event := WSEvent{
		Event:   "BENCH_EVENT",
		Payload: map[string]string{"test": "data"},
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		hub.BroadcastEvent(event)
	}
}
