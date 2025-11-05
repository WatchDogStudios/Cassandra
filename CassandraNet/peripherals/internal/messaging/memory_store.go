package messaging

import (
	"context"
	"sync"
)

// MemoryStore is an in-memory implementation of the messaging Store.
type MemoryStore struct {
	mu      sync.RWMutex
	byTopic map[string][]Message
}

// NewMemoryStore creates an empty MemoryStore.
func NewMemoryStore() *MemoryStore {
	return &MemoryStore{byTopic: make(map[string][]Message)}
}

// Save appends a message to the topic list.
func (m *MemoryStore) Save(_ context.Context, message Message) (Message, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	copy := message
	copy.Attributes = cloneMap(message.Attributes)
	copy.Payload = append([]byte(nil), message.Payload...)
	m.byTopic[message.Topic] = append(m.byTopic[message.Topic], copy)
	return copy, nil
}

// List retrieves messages matching the filter up to the provided limit.
func (m *MemoryStore) List(_ context.Context, filter PullFilter) ([]Message, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	var results []Message
	topicMessages := m.byTopic[filter.Topic]
	for _, message := range topicMessages {
		if filter.TenantID != "" && message.TenantID != filter.TenantID {
			continue
		}
		if filter.ProjectID != "" && message.ProjectID != filter.ProjectID {
			continue
		}
		copy := message
		copy.Attributes = cloneMap(message.Attributes)
		copy.Payload = append([]byte(nil), message.Payload...)
		results = append(results, copy)
		if filter.Limit > 0 && len(results) >= filter.Limit {
			break
		}
	}
	return results, nil
}

// Delete removes a message from a topic.
func (m *MemoryStore) Delete(_ context.Context, topic, messageID string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	messages := m.byTopic[topic]
	for idx, message := range messages {
		if message.MessageID == messageID {
			m.byTopic[topic] = append(messages[:idx], messages[idx+1:]...)
			return nil
		}
	}
	return ErrMessageNotFound
}
