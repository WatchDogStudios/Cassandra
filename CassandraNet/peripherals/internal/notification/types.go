package notification

import "time"

// Channel represents a delivery channel for notifications.
type Channel string

const (
	ChannelEmail   Channel = "email"
	ChannelWebhook Channel = "webhook"
	ChannelInApp   Channel = "in_app"
)

// Message describes an outbound notification request.
type Message struct {
	Channel   Channel        `json:"channel"`
	Recipient string         `json:"recipient"`
	Template  string         `json:"template"`
	Data      map[string]any `json:"data"`
}

// Delivery is the concrete payload delivered to a recipient.
type Delivery struct {
	Channel   Channel   `json:"channel"`
	Recipient string    `json:"recipient"`
	Body      string    `json:"body"`
	SentAt    time.Time `json:"sent_at"`
}
