package ws

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func cleanupHub(hub *Hub) {
	hub.mu.Lock()
	for c := range hub.clients {
		close(c.send)
		delete(hub.clients, c)
	}
	hub.mu.Unlock()
}

func TestHandleMessage_InvalidJSON(t *testing.T) {
	hub := NewHub()
	client := NewClient(hub, nil)

	garbage := []byte("garbage data")
	hub.HandleMessage(client, garbage)

	select {
	case msg := <-client.send:
		var event WSEvent
		err := json.Unmarshal(msg, &event)
		assert.NoError(t, err)
		assert.Equal(t, "ERROR", event.Event)
	default:
		t.Fatal("expected error message on client.send")
	}
}

func TestClient_SetAuth(t *testing.T) {
	hub := NewHub()
	client := NewClient(hub, nil)

	client.SetAuth("user-1", "admin")

	assert.True(t, client.Authenticated())
	assert.Equal(t, "user-1", client.userID)
	assert.Equal(t, "admin", client.role)
}

func TestClient_SetAuthDeadline_Expired(t *testing.T) {
	hub := NewHub()
	client := NewClient(hub, nil)

	expired := false
	client.SetAuthDeadline(50*time.Millisecond, func() {
		expired = true
	})

	time.Sleep(100 * time.Millisecond)
	assert.True(t, expired)
}

func TestClient_SetAuthDeadline_CancelTimer(t *testing.T) {
	hub := NewHub()
	client := NewClient(hub, nil)

	deadlineCalled := false
	client.SetAuthDeadline(5*time.Second, func() {
		deadlineCalled = true
	})

	client.SetAuth("user-2", "admin")

	time.Sleep(100 * time.Millisecond)
	assert.False(t, deadlineCalled)
}

func TestHandleMessage_AUTH(t *testing.T) {
	hub := NewHub()
	var receivedAction Action
	client := NewClient(hub, nil)

	hub.SetActionHandler(func(c *Client, a Action) {
		receivedAction = a
	})

	authPayload := AuthPayload{Token: "test-jwt"}
	payload, _ := json.Marshal(authPayload)
	action := Action{
		Action:  "AUTH",
		Payload: payload,
	}
	msg, _ := json.Marshal(action)
	hub.HandleMessage(client, msg)

	assert.Equal(t, "AUTH", receivedAction.Action)
}

func TestHandleMessage_PING(t *testing.T) {
	hub := NewHub()
	var receivedAction Action
	client := NewClient(hub, nil)

	hub.SetActionHandler(func(c *Client, a Action) {
		receivedAction = a
	})

	action := Action{
		Action:  "PING",
		Payload: json.RawMessage(`{}`),
	}
	msg, _ := json.Marshal(action)
	hub.HandleMessage(client, msg)

	assert.Equal(t, "PING", receivedAction.Action)
}

func TestHandleMessage_CREATE_EDGE(t *testing.T) {
	hub := NewHub()
	var receivedAction Action
	client := NewClient(hub, nil)

	hub.SetActionHandler(func(c *Client, a Action) {
		receivedAction = a
	})

	edgePayload := CreateEdgePayload{
		SourceID:     "src-1",
		TargetID:     "tgt-1",
		RelationType: "reference",
	}
	payload, _ := json.Marshal(edgePayload)
	action := Action{
		Action:  "CREATE_EDGE",
		Payload: payload,
	}
	msg, _ := json.Marshal(action)
	hub.HandleMessage(client, msg)

	assert.Equal(t, "CREATE_EDGE", receivedAction.Action)
}

func TestHandleMessage_DELETE_EDGE(t *testing.T) {
	hub := NewHub()
	var receivedAction Action
	client := NewClient(hub, nil)

	hub.SetActionHandler(func(c *Client, a Action) {
		receivedAction = a
	})

	edgePayload := DeleteEdgePayload{
		SourceID: "src-1",
		TargetID: "tgt-1",
	}
	payload, _ := json.Marshal(edgePayload)
	action := Action{
		Action:  "DELETE_EDGE",
		Payload: payload,
	}
	msg, _ := json.Marshal(action)
	hub.HandleMessage(client, msg)

	assert.Equal(t, "DELETE_EDGE", receivedAction.Action)
}

func TestHub_RegisterUnregister(t *testing.T) {
	hub := NewHub()
	go hub.Run()

	client := NewClient(hub, nil)

	hub.Register(client)
	time.Sleep(50 * time.Millisecond)

	assert.Equal(t, 1, hub.ClientCount())

	hub.Unregister(client)
	time.Sleep(50 * time.Millisecond)

	assert.Equal(t, 0, hub.ClientCount())

	cleanupHub(hub)
}

func TestHub_BroadcastEvent(t *testing.T) {
	hub := NewHub()
	go hub.Run()

	client1 := NewClient(hub, nil)
	client2 := NewClient(hub, nil)

	hub.Register(client1)
	hub.Register(client2)
	time.Sleep(50 * time.Millisecond)

	event := WSEvent{
		Event:   "TEST_EVENT",
		Payload: map[string]string{"message": "hello"},
	}
	hub.BroadcastEvent(event)

	time.Sleep(50 * time.Millisecond)

	var msg1, msg2 []byte
	select {
	case msg1 = <-client1.send:
	default:
	}
	select {
	case msg2 = <-client2.send:
	default:
	}

	assert.NotNil(t, msg1)
	assert.NotNil(t, msg2)

	cleanupHub(hub)
}

func TestHub_BroadcastEvent_SlowClientDropped(t *testing.T) {
	hub := NewHub()
	go hub.Run()

	client1 := NewClient(hub, nil)
	client1.send = make(chan []byte, 1)
	client1.send <- []byte("blocking message")

	client2 := NewClient(hub, nil)

	hub.Register(client1)
	hub.Register(client2)
	time.Sleep(50 * time.Millisecond)

	event := WSEvent{
		Event:   "TEST_EVENT",
		Payload: map[string]string{"message": "hello"},
	}
	hub.BroadcastEvent(event)

	time.Sleep(50 * time.Millisecond)

	var msg2 []byte
	select {
	case msg2 = <-client2.send:
	default:
	}

	assert.NotNil(t, msg2)

	cleanupHub(hub)
}

func TestClient_Send_BufferFull(t *testing.T) {
	hub := NewHub()
	client := NewClient(hub, nil)
	client.send = make(chan []byte, 1)
	client.send <- []byte("blocking")

	done := make(chan bool, 1)
	go func() {
		client.Send([]byte("test message"))
		done <- true
	}()

	select {
	case <-done:
	case <-time.After(100 * time.Millisecond):
		t.Fatal("Send should not block when buffer is full")
	}

	close(client.send)
}
