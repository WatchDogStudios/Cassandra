package ugcworker

import (
	"strings"
	"time"
)

// ModerationPolicy holds simple rules for content moderation.
type ModerationPolicy struct {
	banned []string
}

// NewModerationPolicy constructs a policy with the provided banned terms.
func NewModerationPolicy(banned []string) ModerationPolicy {
	normalized := make([]string, 0, len(banned))
	for _, term := range banned {
		term = strings.TrimSpace(term)
		if term == "" {
			continue
		}
		normalized = append(normalized, strings.ToLower(term))
	}
	return ModerationPolicy{banned: normalized}
}

// Evaluate produces a moderation result for the given job.
func (p ModerationPolicy) Evaluate(job Job) Result {
	lower := strings.ToLower(job.Body)
	for _, banned := range p.banned {
		if strings.Contains(lower, banned) {
			return Result{
				Job:         job,
				Decision:    DecisionFlagged,
				Reason:      "contains banned term: " + banned,
				ProcessedAt: nowUTC(),
			}
		}
	}
	return Result{
		Job:         job,
		Decision:    DecisionApproved,
		Reason:      "passed automated moderation",
		ProcessedAt: nowUTC(),
	}
}

var nowUTC = func() time.Time { return time.Now().UTC() }
