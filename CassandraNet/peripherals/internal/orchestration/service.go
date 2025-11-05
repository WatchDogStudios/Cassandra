package orchestration

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"time"
)

// ErrAssignmentNotFound indicates the requested assignment is not present in the store.
var ErrAssignmentNotFound = errors.New("orchestration: assignment not found")

// Store encapsulates persistence for assignments.
type Store interface {
	CreateAssignment(ctx context.Context, assignment Assignment) (Assignment, error)
	UpdateAssignment(ctx context.Context, id string, status Status, message string, updatedAt time.Time) (Assignment, error)
	ListAssignments(ctx context.Context, filter ListAssignmentsFilter) ([]Assignment, error)
}

// Clock provides time keeping; overridable for tests.
type Clock interface {
	Now() time.Time
}

type systemClock struct{}

func (systemClock) Now() time.Time { return time.Now().UTC() }

// Service performs orchestration tasks backed by a Store.
type Service struct {
	store Store
	clock Clock
}

// NewService constructs a Service instance.
func NewService(store Store, clock Clock) *Service {
	if clock == nil {
		clock = systemClock{}
	}
	return &Service{store: store, clock: clock}
}

// AssignWork creates a new assignment for the provided agent/workload pair.
func (s *Service) AssignWork(ctx context.Context, req AssignRequest) (Assignment, error) {
	if req.AgentID == "" || req.WorkloadID == "" {
		return Assignment{}, errors.New("agent_id and workload_id required")
	}
	assignment := Assignment{
		AssignmentID:  newIdentifier(),
		AgentID:       req.AgentID,
		WorkloadID:    req.WorkloadID,
		TenantID:      req.TenantID,
		ProjectID:     req.ProjectID,
		Status:        StatusPending,
		StatusMessage: "queued",
		Metadata:      cloneMetadata(req.Metadata),
	}
	now := s.clock.Now()
	assignment.CreatedAt = now
	assignment.UpdatedAt = now
	created, err := s.store.CreateAssignment(ctx, assignment)
	if err != nil {
		return Assignment{}, err
	}
	return created, nil
}

// UpdateStatus applies a status transition on an assignment.
func (s *Service) UpdateStatus(ctx context.Context, req UpdateStatusRequest) (Assignment, error) {
	if req.AssignmentID == "" {
		return Assignment{}, errors.New("assignment_id required")
	}
	if req.Status == "" {
		return Assignment{}, errors.New("status required")
	}
	updated, err := s.store.UpdateAssignment(ctx, req.AssignmentID, req.Status, req.StatusMessage, s.clock.Now())
	if err != nil {
		return Assignment{}, err
	}
	return updated, nil
}

// ListAssignments returns assignments matching the filter.
func (s *Service) ListAssignments(ctx context.Context, filter ListAssignmentsFilter) ([]Assignment, error) {
	assignments, err := s.store.ListAssignments(ctx, filter)
	if err != nil {
		return nil, err
	}
	return assignments, nil
}

func cloneMetadata(in map[string]string) map[string]string {
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
