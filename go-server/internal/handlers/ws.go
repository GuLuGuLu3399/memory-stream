package handlers

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/services"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/ws"
	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
)

const (
	// 心跳超时：客户端必须在此时间内发送Pong，否则连接断开
	pongWait = 60 * time.Second
	// 发送心跳间隔
	pingInterval = 30 * time.Second
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     checkWebSocketOrigin,
}

// checkWebSocketOrigin validates incoming WebSocket upgrade requests against allowed origins.
// Supports local development (Vite), Tauri desktop, and production deployments.
func checkWebSocketOrigin(r *http.Request) bool {
	origin := r.Header.Get("Origin")
	
	// Allowed origins: development servers, Tauri, and production domains
	allowedOrigins := []string{
		"http://localhost:5173",   // Vite dev server
		"http://localhost:1420",   // Tauri dev server
		"http://localhost:4173",   // Vite preview
		"https://tauri.localhost",
		"http://tauri.localhost",
	}
	
	for _, allowed := range allowedOrigins {
		if origin == allowed {
			return true
		}
	}
	
	logger.Log.Warnf("[WS] rejected upgrade from disallowed origin: %s", origin)
	return false
}

// extractBearerToken 从请求头或URL Query中提取Token
// 优先级：Authorization header > URL query parameter
func extractBearerToken(r *http.Request) string {
	// 1. 尝试从 Authorization header 提取 (标准方式)
	auth := r.Header.Get("Authorization")
	if auth != "" {
		parts := strings.SplitN(auth, " ", 2)
		if len(parts) == 2 && strings.EqualFold(parts[0], "bearer") {
			return strings.TrimSpace(parts[1])
		}
	}
	
	// 2. 降阶到 URL Query (WebSocket API原生兼容性)
	return r.URL.Query().Get("token")
}

// HandleWS 处理WebSocket握手与升级
// 【第一纪元】在HTTP升级握手阶段直接验证Token，拦截伪造者于门外
func HandleWS(c *gin.Context, hub *ws.Hub, authSvc *services.AuthService) {
	// ══════════════════════════════════════════════════════════════════
	// 阶段1：Token提取与验证（握手阶段拦截）
	// ══════════════════════════════════════════════════════════════════
	tokenString := extractBearerToken(c.Request)
	if tokenString == "" {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "缺少认证令牌"})
		logger.Log.Warnf("[WS] connection attempt without token from %s", c.ClientIP())
		return
	}

	// Token校验：无效Token直接拒绝，不给升级机会
	userID, role, err := authSvc.ParseAccessToken(tokenString)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "令牌无效或已过期"})
		logger.Log.Warnf("[WS] auth failed for %s: %v", c.ClientIP(), err)
		return
	}

	// ══════════════════════════════════════════════════════════════════
	// 阶段2：正式升级连接（已验证的客户端）
	// ══════════════════════════════════════════════════════════════════
	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		logger.Log.Errorf("[WS] upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	// ══════════════════════════════════════════════════════════════════
	// 阶段3：创建客户端对象与心跳管理
	// ══════════════════════════════════════════════════════════════════
	client := ws.NewClient(hub, conn)
	client.SetAuth(userID, role)

	// 设置心跳超时：如果60秒内没有收到Pong，连接自动断开
	conn.SetReadDeadline(time.Now().Add(pongWait))
	
	// Pong处理器：收到客户端的心跳回应，续命
	conn.SetPongHandler(func(string) error {
		logger.Log.Debugf("[WS] pong from %s", userID)
		conn.SetReadDeadline(time.Now().Add(pongWait))
		return nil
	})

	hub.Register(client)
	
	logger.Log.Infof("[WS] authenticated: user=%s role=%s from=%s", 
		userID, role, c.ClientIP())

	// ══════════════════════════════════════════════════════════════════
	// 阶段4：启动读写泵与心跳发送器
	// ══════════════════════════════════════════════════════════════════
	go client.WritePump()
	go client.ReadPump()
	go heartbeatSender(conn, userID, pingInterval)
}

// heartbeatSender 周期性向客户端发送Ping心跳
// 如果客户端在pongWait内不回应Pong，连接会因SetReadDeadline而断开
func heartbeatSender(conn *websocket.Conn, userID string, interval time.Duration) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for range ticker.C {
		if err := conn.WriteControl(
			websocket.PingMessage,
			[]byte{},
			time.Now().Add(5*time.Second),
		); err != nil {
			logger.Log.Warnf("[WS] heartbeat send failed for %s: %v", userID, err)
			return
		}
		logger.Log.Debugf("[WS] ping sent to %s", userID)
	}
}

// SetupWSHandlers registers action handlers for WebSocket events (AUTH, CREATE_EDGE, DELETE_EDGE, PING).
func SetupWSHandlers(hub *ws.Hub, edgeSvc *services.EdgeService, authSvc *services.AuthService) {
	hub.SetActionHandler(func(client *ws.Client, action ws.Action) {
		switch action.Action {
		// 注意：由于Token已在握手阶段验证，这里的AUTH消息不再必需
		// 但保留以兼容旧客户端
		case "AUTH":
			// 握手时已验证，这里只是日志记录
			logger.Log.Debugf("[WS] client reaffirms auth")
			
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
			// 心跳应答：客户端主动Ping，服务端回Pong
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

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := edgeSvc.CreateEdge(ctx, req.SourceID, req.TargetID, req.RelationType); err != nil {
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

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := edgeSvc.DeleteEdge(ctx, req.SourceID, req.TargetID); err != nil {
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
