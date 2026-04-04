package ws

import "encoding/json"

type Action struct {
	Action  string          `json:"action"`
	Payload json.RawMessage `json:"payload"`
}

type CreateEdgePayload struct {
	SourceID     string `json:"source_id"`
	TargetID     string `json:"target_id"`
	RelationType string `json:"relation_type"`
}

type DeleteEdgePayload struct {
	SourceID string `json:"source_id"`
	TargetID string `json:"target_id"`
}

type AuthPayload struct {
	Token string `json:"token"`
}

type WSEvent struct {
	Event   string      `json:"event"`
	Payload interface{} `json:"payload"`
}

type LayoutUpdatedPayload []LayoutItem

type LayoutItem struct {
	ID string  `json:"id"`
	X  float64 `json:"x"`
	Y  float64 `json:"y"`
}

type ErrorPayload struct {
	Message string `json:"message"`
}

// CardEventPayload 卡片变更事件载荷
type CardEventPayload struct {
	CardID     string `json:"card_id"`
	Title      string `json:"title,omitempty"`
	Excerpt    string `json:"excerpt,omitempty"`
	CategoryID string `json:"category_id,omitempty"`
}

// EdgeEventPayload 边变更事件载荷（复用于 EDGE_CREATED / EDGE_DELETED）
type EdgeEventPayload struct {
	SourceID     string `json:"source_id"`
	TargetID     string `json:"target_id"`
	RelationType string `json:"relation_type,omitempty"`
}
