package ugc

import (
	"context"
	"sync"
	"time"
)

// MemoryStore implements Store using an in-memory map.
type MemoryStore struct {
	mu   sync.RWMutex
	byID map[string]Content
}

// NewMemoryStore constructs an empty memory store.
func NewMemoryStore() *MemoryStore {
	return &MemoryStore{byID: make(map[string]Content)}
}

// Create inserts a new content record.
func (m *MemoryStore) Create(_ context.Context, content Content) (Content, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	copy := content
	copy.Labels = cloneMap(content.Labels)
	copy.Attributes = cloneMap(content.Attributes)
	m.byID[copy.ContentID] = copy
	return copy, nil
}

// UpdateState updates the moderation state for content.
func (m *MemoryStore) UpdateState(_ context.Context, id string, state State, reason string, updatedAt time.Time) (Content, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	existing, ok := m.byID[id]
	if !ok {
		return Content{}, ErrContentNotFound
	}
	existing.State = state
	existing.Reason = reason
	existing.UpdatedAt = updatedAt
	m.byID[id] = existing
	return existing, nil
}

// List returns content records matching filter options.
func (m *MemoryStore) List(_ context.Context, filter ListFilter) ([]Content, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	var items []Content
	for _, content := range m.byID {
		if filter.TenantID != "" && content.TenantID != filter.TenantID {
			continue
		}
		if filter.ProjectID != "" && content.ProjectID != filter.ProjectID {
			continue
		}
		if filter.State != "" && content.State != filter.State {
			continue
		}
		copy := content
		copy.Labels = cloneMap(content.Labels)
		copy.Attributes = cloneMap(content.Attributes)
		items = append(items, copy)
	}
	return items, nil
}
