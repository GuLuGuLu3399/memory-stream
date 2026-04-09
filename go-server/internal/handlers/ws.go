package handlers

import (
	"encoding/json"
	"net/http"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
)

// Allowed origins for WebSocket connections
var allowedOrigins = map[string]bool{
	"http://localhost:5173":   true, // web-reader dev
	"http://localhost:1420":   true, // admin-tauri dev
	"http://localhost:4173":   true, // vite preview
	"https://tauri.localhost": true, // tauri production
	"http://tauri.localhost":  true, // tauri production (some versions)
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		origin := r.Header.Get("Origin")
		if origin == "" {
			// Allow connections without Origin header (e.g., direct WebSocket clients)
			return true
		}
		allowed, exists := allowedOrigins[origin]
		return exists && allowed
	},
}

// HandleWS upgrades an HTTP connection to WebSocket and starts the client pumps.
func HandleWS(c *gin.Context, hub *ws.Hub) {
	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		logger.Log.Errorf("[WS] upgrade failed: %v", err)
		return
	}

	client := ws.NewClient(hub, conn)

	// 所有客户端必须通过 AUTH 消息认证，不再接受 URL Query Token
	client.SetAuthDeadline(3*time.Second, func() {
		client.SendError("AUTH", "认证超时")
		client.Close()
	})

	hub.Register(client)
	go client.WritePump()
	go client.ReadPump()
}

// SetupWSHandlers registers action handlers for WebSocket events (AUTH, CREATE_EDGE, DELETE_EDGE, PING).
func SetupWSHandlers(hub *ws.Hub, edgeSvc *services.EdgeService, authSvc *services.AuthService) {
	hub.SetActionHandler(func(client *ws.Client, action ws.Action) {
		switch action.Action {
		case "AUTH":
			handleWSAuth(authSvc, client, action.Payload)
		case "CREATE_EDGE":
			if !client.Authenticated() {
				client.SendError("CREATE_EDGE", "未认证")
				return
			}
			handleWSCreateEdge(hub, edgeSvc, action.Payload)
		case "DELETE_EDGE":
			if !client.Authenticated() {
				client.SendError("DELETE_EDGE", "未认证")
				return
			}
			handleWSDeleteEdge(hub, edgeSvc, action.Payload)
		case "PING":
			// 心跳延迟检测 — 回送 PONG
			data, _ := json.Marshal(ws.WSEvent{Event: "PONG"})
			client.Send(data)
		default:
			hub.BroadcastEvent(ws.WSEvent{
				Event:   "ERROR",
				Payload: ws.ErrorPayload{Message: "unknown action: " + action.Action},
			})
		}
	})
}

func handleWSAuth(authSvc *services.AuthService, client *ws.Client, payload json.RawMessage) {
	var req ws.AuthPayload
	if err := json.Unmarshal(payload, &req); err != nil {
		client.SendError("AUTH", "invalid AUTH payload")
		return
	}

	userID, role, err := authSvc.ParseAccessToken(req.Token)
	if err != nil {
		client.SendError("AUTH", "token无效或已过期")
		return
	}

	client.SetAuth(userID, role)

	data, _ := json.Marshal(ws.WSEvent{
		Event:   "AUTH_OK",
		Payload: map[string]string{"user_id": userID, "role": role},
	})
	client.Send(data)

	logger.Log.Infof("[WS] client authenticated: user=%s role=%s", userID, role)
}

func handleWSCreateEdge(hub *ws.Hub, edgeSvc *services.EdgeService, payload json.RawMessage) {
	var req ws.CreateEdgePayload
	if err := json.Unmarshal(payload, &req); err != nil {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "invalid CREATE_EDGE payload"},
		})
		return
	}

	if req.SourceID == "" || req.TargetID == "" {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "source_id and target_id are required"},
		})
		return
	}

	if err := edgeSvc.CreateEdge(req.SourceID, req.TargetID, req.RelationType); err != nil {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "failed to create edge: " + err.Error()},
		})
		return
	}

	hub.BroadcastEvent(ws.WSEvent{
		Event: "EDGE_CREATED",
		Payload: gin.H{
			"source_id":     req.SourceID,
			"target_id":     req.TargetID,
			"relation_type": req.RelationType,
		},
	})

	logger.Log.Infof("[WS] edge created: %s -> %s (%s)", req.SourceID, req.TargetID, req.RelationType)
}

func handleWSDeleteEdge(hub *ws.Hub, edgeSvc *services.EdgeService, payload json.RawMessage) {
	var req ws.DeleteEdgePayload
	if err := json.Unmarshal(payload, &req); err != nil {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "invalid DELETE_EDGE payload"},
		})
		return
	}

	if req.SourceID == "" || req.TargetID == "" {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "source_id and target_id are required"},
		})
		return
	}

	if err := edgeSvc.DeleteEdge(req.SourceID, req.TargetID); err != nil {
		hub.BroadcastEvent(ws.WSEvent{
			Event:   "ERROR",
			Payload: ws.ErrorPayload{Message: "failed to delete edge: " + err.Error()},
		})
		return
	}

	hub.BroadcastEvent(ws.WSEvent{
		Event: "EDGE_DELETED",
		Payload: gin.H{
			"source_id": req.SourceID,
			"target_id": req.TargetID,
		},
	})

	logger.Log.Infof("[WS] edge deleted: %s -> %s", req.SourceID, req.TargetID)
}
