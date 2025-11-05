package orchestration

import "time"

// Status describes the lifecycle for an assignment. Mirrors proto WorkStatus values.
type Status string

const (
	StatusPending   Status = "pending"
	StatusAssigned  Status = "assigned"
	StatusRunning   Status = "in_progress"
	StatusCompleted Status = "completed"
	StatusFailed    Status = "failed"
	StatusCancelled Status = "cancelled"
)

// Assignment models a unit of work targeting an agent.
type Assignment struct {
	AssignmentID  string            `json:"assignment_id"`
	AgentID       string            `json:"agent_id"`
	WorkloadID    string            `json:"workload_id"`
	TenantID      string            `json:"tenant_id"`
	ProjectID     string            `json:"project_id"`
	Status        Status            `json:"status"`
	StatusMessage string            `json:"status_message,omitempty"`
	CreatedAt     time.Time         `json:"created_at"`
	UpdatedAt     time.Time         `json:"updated_at"`
	Metadata      map[string]string `json:"metadata,omitempty"`
}

// AssignRequest is the payload required to create an assignment.
type AssignRequest struct {
	AgentID    string
	WorkloadID string
	TenantID   string
	ProjectID  string
	Metadata   map[string]string
}

// UpdateStatusRequest describes a status transition.
type UpdateStatusRequest struct {
	AssignmentID  string
	Status        Status
	StatusMessage string
}

// ListAssignmentsFilter contains filters applied when listing assignments.
type ListAssignmentsFilter struct {
	AgentID   string
	TenantID  string
	ProjectID string
	Status    Status
}
