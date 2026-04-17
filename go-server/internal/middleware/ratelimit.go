package middleware

import (
	"fmt"
	"sync"
	"time"
)

type ViewRateLimiter struct {
	mu       sync.RWMutex
	store    map[string]time.Time
	ttl      time.Duration
	stopCh   chan struct{}
	stopOnce sync.Once
}

func NewViewRateLimiter() *ViewRateLimiter {
	vl := &ViewRateLimiter{
		store:  make(map[string]time.Time),
		ttl:    10 * time.Minute,
		stopCh: make(chan struct{}),
	}
	go vl.cleanup()
	return vl
}

func (vl *ViewRateLimiter) Allow(ip, cardID string) bool {
	key := fmt.Sprintf("view_limit:%s:%s", ip, cardID)

	vl.mu.RLock()
	expireAt, exists := vl.store[key]
	vl.mu.RUnlock()

	if exists && time.Now().Before(expireAt) {
		return false
	}

	vl.mu.Lock()
	vl.store[key] = time.Now().Add(vl.ttl)
	vl.mu.Unlock()
	return true
}

func (vl *ViewRateLimiter) Stop() {
	vl.stopOnce.Do(func() { close(vl.stopCh) })
}

func (vl *ViewRateLimiter) cleanup() {
	ticker := time.NewTicker(time.Minute)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			vl.mu.Lock()
			now := time.Now()
			for k, v := range vl.store {
				if now.After(v) {
					delete(vl.store, k)
				}
			}
			vl.mu.Unlock()
		case <-vl.stopCh:
			return
		}
	}
}
