package messaging

import "time"

// Priority reflects delivery urgency for messages.
type Priority string

const (
	PriorityLow    Priority = "low"
	PriorityNormal Priority = "normal"
	PriorityHigh   Priority = "high"
)

// Message encapsulates a single event routed through the messaging service.
type Message struct {
	MessageID   string            `json:"message_id"`
	TenantID    string            `json:"tenant_id"`
	ProjectID   string            `json:"project_id"`
	Topic       string            `json:"topic"`
	Key         string            `json:"key"`
	Payload     []byte            `json:"-"`
	Priority    Priority          `json:"priority"`
	PublishedAt time.Time         `json:"published_at"`
	Attributes  map[string]string `json:"attributes,omitempty"`
}

// PublishRequest collects publish properties from clients.
type PublishRequest struct {
	TenantID   string
	ProjectID  string
	Topic      string
	Key        string
	Payload    []byte
	Priority   Priority
	Attributes map[string]string
}

// PullFilter controls message retrieval.
type PullFilter struct {
	TenantID  string
	ProjectID string
	Topic     string
	Limit     int
}
