package ugc

import "time"

// State captures the moderation status for UGC submissions.
type State string

const (
	StatePending  State = "pending"
	StateApproved State = "approved"
	StateRejected State = "rejected"
	StateArchived State = "archived"
)

// Content represents metadata for a submitted content item.
type Content struct {
	ContentID   string            `json:"content_id"`
	TenantID    string            `json:"tenant_id"`
	ProjectID   string            `json:"project_id"`
	Filename    string            `json:"filename"`
	MimeType    string            `json:"mime_type"`
	SizeBytes   uint64            `json:"size_bytes"`
	State       State             `json:"state"`
	Reason      string            `json:"reason,omitempty"`
	SubmittedAt time.Time         `json:"submitted_at"`
	UpdatedAt   time.Time         `json:"updated_at"`
	Labels      map[string]string `json:"labels,omitempty"`
	Attributes  map[string]string `json:"attributes,omitempty"`
}

// SubmitRequest carries submission metadata.
type SubmitRequest struct {
	ContentID  string
	TenantID   string
	ProjectID  string
	Filename   string
	MimeType   string
	SizeBytes  uint64
	Labels     map[string]string
	Attributes map[string]string
}

// ReviewRequest captures a moderation decision.
type ReviewRequest struct {
	ContentID string
	TenantID  string
	ProjectID string
	State     State
	Reason    string
}

// ListFilter holds filtering options when listing content.
type ListFilter struct {
	TenantID  string
	ProjectID string
	State     State
}
