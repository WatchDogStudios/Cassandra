package ugc

import (
	"context"
	"errors"
	"time"
)

// ErrContentNotFound indicates the content does not exist.
var ErrContentNotFound = errors.New("ugc: content not found")

// Store abstracts persistence for UGC submissions.
type Store interface {
	Create(ctx context.Context, content Content) (Content, error)
	UpdateState(ctx context.Context, id string, state State, reason string, updatedAt time.Time) (Content, error)
	List(ctx context.Context, filter ListFilter) ([]Content, error)
}

// Clock allows deterministic timing in tests.
type Clock interface {
	Now() time.Time
}

type systemClock struct{}

func (systemClock) Now() time.Time { return time.Now().UTC() }

// Service orchestrates moderation actions.
type Service struct {
	store Store
	clock Clock
}

// NewService builds a Service with the provided store.
func NewService(store Store, clock Clock) *Service {
	if clock == nil {
		clock = systemClock{}
	}
	return &Service{store: store, clock: clock}
}

// SubmitContent stores a new submission and returns its metadata.
func (s *Service) SubmitContent(ctx context.Context, req SubmitRequest) (Content, error) {
	if req.ContentID == "" || req.TenantID == "" || req.ProjectID == "" || req.Filename == "" {
		return Content{}, errors.New("content_id, tenant_id, project_id, and filename required")
	}
	content := Content{
		ContentID:  req.ContentID,
		TenantID:   req.TenantID,
		ProjectID:  req.ProjectID,
		Filename:   req.Filename,
		MimeType:   req.MimeType,
		SizeBytes:  req.SizeBytes,
		State:      StatePending,
		Labels:     cloneMap(req.Labels),
		Attributes: cloneMap(req.Attributes),
	}
	now := s.clock.Now()
	content.SubmittedAt = now
	content.UpdatedAt = now
	created, err := s.store.Create(ctx, content)
	if err != nil {
		return Content{}, err
	}
	return created, nil
}

// ReviewContent updates the moderation state for an item.
func (s *Service) ReviewContent(ctx context.Context, req ReviewRequest) (Content, error) {
	if req.ContentID == "" {
		return Content{}, errors.New("content_id required")
	}
	if req.State == "" {
		return Content{}, errors.New("state required")
	}
	updated, err := s.store.UpdateState(ctx, req.ContentID, req.State, req.Reason, s.clock.Now())
	if err != nil {
		return Content{}, err
	}
	return updated, nil
}

// ListContent lists content records using provided filter.
func (s *Service) ListContent(ctx context.Context, filter ListFilter) ([]Content, error) {
	items, err := s.store.List(ctx, filter)
	if err != nil {
		return nil, err
	}
	return items, nil
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
