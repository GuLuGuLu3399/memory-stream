package ws

import (
	"encoding/json"
	"runtime/debug"
	"sync"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/gorilla/websocket"
)

const (
	writeWait  = 10 * time.Second
	pongWait   = 60 * time.Second
	pingPeriod = 30 * time.Second

	maxMessageSize = 4096
)

type Hub struct {
	mu         sync.RWMutex
	clients    map[*Client]bool
	broadcast  chan []byte
	register   chan *Client
	unregister chan *Client
	onAction   func(client *Client, action Action)
}

func NewHub() *Hub {
	return &Hub{
		clients:    make(map[*Client]bool),
		broadcast:  make(chan []byte, 256),
		register:   make(chan *Client),
		unregister: make(chan *Client),
	}
}

func (h *Hub) SetActionHandler(fn func(client *Client, action Action)) {
	h.onAction = fn
}

func (h *Hub) Run() {
	defer func() {
		if r := recover(); r != nil {
			logger.Log.Errorf("[WS Hub] Run panic recovered: %v\n%s", r, debug.Stack())
			// Restart the event loop after panic recovery
			go h.Run()
		}
	}()
	for {
		select {
		case client := <-h.register:
			h.mu.Lock()
			h.clients[client] = true
			h.mu.Unlock()
			logger.Log.Infof("[WS Hub] connected | total: %d | authed: %v", len(h.clients), client.authenticated)
		case client := <-h.unregister:
			h.mu.Lock()
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
			}
			h.mu.Unlock()
			logger.Log.Infof("[WS Hub] disconnected | total: %d", len(h.clients))
		case message := <-h.broadcast:
			h.mu.RLock()
			var slowClients []*Client
			for client := range h.clients {
				select {
				case client.send <- message:
				default:
					slowClients = append(slowClients, client)
				}
			}
			h.mu.RUnlock()
			if len(slowClients) > 0 {
				h.mu.Lock()
				for _, c := range slowClients {
					if _, ok := h.clients[c]; ok {
						delete(h.clients, c)
						close(c.send)
					}
				}
				h.mu.Unlock()
			}
		}
	}
}

func (h *Hub) Register(client *Client)   { h.register <- client }
func (h *Hub) Unregister(client *Client) { h.unregister <- client }

func (h *Hub) BroadcastEvent(event WSEvent) {
	data, err := json.Marshal(event)
	if err != nil {
		logger.Log.Errorf("[WS Hub] marshal error for event %s: %v", event.Event, err)
		// 尝试降级为最小化错误事件，避免完全丢失
		fallback, fbErr := json.Marshal(WSEvent{
			Event:   event.Event,
			Payload: map[string]string{"error": "marshal failed"},
		})
		if fbErr != nil {
			logger.Log.Errorf("[WS Hub] fallback marshal also failed: %v", fbErr)
			return
		}
		data = fallback
	}
	h.broadcast <- data
}

func (h *Hub) HandleMessage(client *Client, msg []byte) {
	var action Action
	if err := json.Unmarshal(msg, &action); err != nil {
		errData, _ := json.Marshal(WSEvent{Event: "ERROR", Payload: ErrorPayload{Message: "invalid JSON"}})
		client.Send(errData)
		return
	}
	if h.onAction != nil {
		h.onAction(client, action)
	}
}

func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}

type Client struct {
	hub           *Hub
	conn          *websocket.Conn
	send          chan []byte
	userID        string
	role          string
	authenticated bool
	authTimer     *time.Timer
}

func NewClient(hub *Hub, conn *websocket.Conn) *Client {
	return &Client{hub: hub, conn: conn, send: make(chan []byte, 256)}
}

func (c *Client) SetAuth(userID, role string) {
	c.userID = userID
	c.role = role
	c.authenticated = true
	if c.authTimer != nil {
		c.authTimer.Stop()
		c.authTimer = nil
	}
}

func (c *Client) SetAuthDeadline(d time.Duration, onExpired func()) {
	c.authTimer = time.AfterFunc(d, onExpired)
}

func (c *Client) SendError(action, msg string) {
	defer func() {
		if r := recover(); r != nil {
			logger.Log.Debugf("[WS] SendError on closed channel (action=%s): %v", action, r)
		}
	}()
	data, _ := json.Marshal(WSEvent{
		Event:   "ERROR",
		Payload: ErrorPayload{Message: msg},
	})
	select {
	case c.send <- data:
	default:
	}
}

func (c *Client) Close() {
	err := c.conn.Close()
	if err != nil {
		logger.Log.Errorf("[WS Hub] close conn error: %v", err)
		return
	}
}

func (c *Client) Authenticated() bool {
	return c.authenticated
}

func (c *Client) Send(data []byte) {
	defer func() {
		if r := recover(); r != nil {
			logger.Log.Debug("[WS] Send on closed channel, dropping message")
		}
	}()
	select {
	case c.send <- data:
	default:
	}
}

func (c *Client) ReadPump() {
	defer func() {
		if r := recover(); r != nil {
			logger.Log.Errorf("[WS] ReadPump panic recovered: %v\n%s", r, debug.Stack())
		}
		c.hub.Unregister(c)
		err := c.conn.Close()
		if err != nil {
			logger.Log.Errorf("[WS Hub] close conn error: %v", err)
			return
		}
	}()

	c.conn.SetReadLimit(maxMessageSize)
	err := c.conn.SetReadDeadline(time.Now().Add(pongWait))
	if err != nil {
		logger.Log.Errorf("[WS Hub] set read deadline error: %v", err)
		return
	}
	c.conn.SetPongHandler(func(string) error {
		err := c.conn.SetReadDeadline(time.Now().Add(pongWait))
		if err != nil {
			logger.Log.Errorf("[WS Hub] set read deadline error: %v", err)
			return err
		}
		return nil
	})

	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			break
		}
		c.hub.HandleMessage(c, message)
	}
}

func (c *Client) WritePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		if r := recover(); r != nil {
			logger.Log.Errorf("[WS] WritePump panic recovered: %v\n%s", r, debug.Stack())
		}
		ticker.Stop()
		err := c.conn.Close()
		if err != nil {
			logger.Log.Errorf("[WS Hub] close conn error: %v", err)
			return
		}
	}()

	for {
		select {
		case message, ok := <-c.send:
			err := c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err != nil {
				logger.Log.Errorf("[WS Hub] set write deadline error: %v", err)
				return
			}
			if !ok {
				err := c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				if err != nil {
					logger.Log.Errorf("[WS Hub] close write error: %v", err)
					return
				}
				return
			}
			if err := c.conn.WriteMessage(websocket.TextMessage, message); err != nil {
				return
			}
		case <-ticker.C:
			err := c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err != nil {
				logger.Log.Errorf("[WS Hub] set write deadline error: %v", err)
				return
			}
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}
