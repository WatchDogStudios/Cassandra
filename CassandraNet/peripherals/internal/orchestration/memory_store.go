package orchestration

import (
	"context"
	"sync"
	"time"
)

// MemoryStore provides an in-memory implementation of the Store interface.
type MemoryStore struct {
	mu          sync.RWMutex
	assignments map[string]Assignment
}

// NewMemoryStore constructs an empty MemoryStore.
func NewMemoryStore() *MemoryStore {
	return &MemoryStore{assignments: make(map[string]Assignment)}
}

// CreateAssignment inserts a new assignment record.
func (m *MemoryStore) CreateAssignment(_ context.Context, assignment Assignment) (Assignment, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	copy := assignment
	copy.Metadata = cloneMetadata(assignment.Metadata)
	m.assignments[copy.AssignmentID] = copy
	return copy, nil
}

// UpdateAssignment updates status metadata for a given assignment.
func (m *MemoryStore) UpdateAssignment(_ context.Context, id string, status Status, message string, updatedAt time.Time) (Assignment, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	existing, ok := m.assignments[id]
	if !ok {
		return Assignment{}, ErrAssignmentNotFound
	}
	existing.Status = status
	existing.StatusMessage = message
	existing.UpdatedAt = updatedAt
	m.assignments[id] = existing
	return existing, nil
}

// ListAssignments returns assignments matching the provided filter.
func (m *MemoryStore) ListAssignments(_ context.Context, filter ListAssignmentsFilter) ([]Assignment, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	var results []Assignment
	for _, assignment := range m.assignments {
		if filter.AgentID != "" && assignment.AgentID != filter.AgentID {
			continue
		}
		if filter.TenantID != "" && assignment.TenantID != filter.TenantID {
			continue
		}
		if filter.ProjectID != "" && assignment.ProjectID != filter.ProjectID {
			continue
		}
		if filter.Status != "" && assignment.Status != filter.Status {
			continue
		}
		copy := assignment
		copy.Metadata = cloneMetadata(assignment.Metadata)
		results = append(results, copy)
	}
	return results, nil
}
