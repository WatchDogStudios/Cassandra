package messaging

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"strings"
	"time"
)

// ErrMessageNotFound is returned when an ack references a non-existent message.
var ErrMessageNotFound = errors.New("messaging: message not found")

// Store abstracts persistence for messaging workloads.
type Store interface {
	Save(ctx context.Context, message Message) (Message, error)
	List(ctx context.Context, filter PullFilter) ([]Message, error)
	Delete(ctx context.Context, topic, messageID string) error
}

// Clock enables deterministic timing in tests.
type Clock interface {
	Now() time.Time
}

type systemClock struct{}

func (systemClock) Now() time.Time { return time.Now().UTC() }

// Service coordinates messaging workflows.
type Service struct {
	store Store
	clock Clock
}

// NewService constructs a Service.
func NewService(store Store, clock Clock) *Service {
	if clock == nil {
		clock = systemClock{}
	}
	return &Service{store: store, clock: clock}
}

// Publish enqueues a message.
func (s *Service) Publish(ctx context.Context, req PublishRequest) (Message, error) {
	if req.TenantID == "" || req.ProjectID == "" || req.Topic == "" {
		return Message{}, errors.New("tenant_id, project_id, and topic required")
	}
	priority := req.Priority
	if priority == "" {
		priority = PriorityNormal
	}
	message := Message{
		MessageID:   newIdentifier(),
		TenantID:    req.TenantID,
		ProjectID:   req.ProjectID,
		Topic:       req.Topic,
		Key:         req.Key,
		Payload:     append([]byte(nil), req.Payload...),
		Priority:    priority,
		PublishedAt: s.clock.Now(),
		Attributes:  cloneMap(req.Attributes),
	}
	saved, err := s.store.Save(ctx, message)
	if err != nil {
		return Message{}, err
	}
	return saved, nil
}

// Pull retrieves messages matching the filter up to the provided limit.
func (s *Service) Pull(ctx context.Context, filter PullFilter) ([]Message, error) {
	if filter.Topic == "" {
		return nil, errors.New("topic required")
	}
	if filter.Limit <= 0 {
		filter.Limit = 10
	}
	messages, err := s.store.List(ctx, filter)
	if err != nil {
		return nil, err
	}
	// Ensure payload slices are not shared with store state.
	for i := range messages {
		messages[i].Payload = append([]byte(nil), messages[i].Payload...)
	}
	return messages, nil
}

// Ack removes a message after successful processing.
func (s *Service) Ack(ctx context.Context, topic, messageID string) error {
	if topic == "" || messageID == "" {
		return errors.New("topic and message_id required")
	}
	return s.store.Delete(ctx, topic, messageID)
}

// EncodePayloadBase64 creates a base64 representation of message payloads.
func EncodePayloadBase64(message Message) string {
	if len(message.Payload) == 0 {
		return ""
	}
	return base64.StdEncoding.EncodeToString(message.Payload)
}

// DecodePayloadBase64 decodes a base64 string into bytes.
func DecodePayloadBase64(value string) ([]byte, error) {
	if value == "" {
		return nil, nil
	}
	return base64.StdEncoding.DecodeString(value)
}

// ParsePriority converts user supplied strings into Priority values.
func ParsePriority(value string) (Priority, error) {
	switch strings.ToLower(value) {
	case "", "normal":
		return PriorityNormal, nil
	case "low":
		return PriorityLow, nil
	case "high":
		return PriorityHigh, nil
	default:
		return "", errors.New("unknown priority")
	}
}

func cloneMap(in map[string]string) map[string]string {
	if len(in) == 0 {
		return nil
	}
	out := make(map[string]string, len(in))
	for k, v := range in {
		out[k] = v
	}
	return out
}

func newIdentifier() string {
	buf := make([]byte, 16)
	if _, err := rand.Read(buf); err != nil {
		return hex.EncodeToString([]byte(time.Now().UTC().Format("20060102150405.000")))
	}
	return hex.EncodeToString(buf)
}
