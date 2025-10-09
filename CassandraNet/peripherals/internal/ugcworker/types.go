package ugcworker

import "time"

// Job represents a moderation request for user-generated content.
type Job struct {
	ContentID string    `json:"content_id"`
	AuthorID  string    `json:"author_id"`
	Body      string    `json:"body"`
	Submitted time.Time `json:"submitted"`
}

// Decision captures the moderation outcome.
type Decision string

const (
	DecisionApproved Decision = "approved"
	DecisionFlagged  Decision = "flagged"
)

// Result represents a moderation verdict for a job.
type Result struct {
	Job         Job       `json:"job"`
	Decision    Decision  `json:"decision"`
	Reason      string    `json:"reason"`
	ProcessedAt time.Time `json:"processed_at"`
}
