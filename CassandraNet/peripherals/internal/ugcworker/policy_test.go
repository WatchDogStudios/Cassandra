package ugcworker

import (
	"testing"
	"time"
)

func TestModerationPolicyEvaluate(t *testing.T) {
	policy := NewModerationPolicy([]string{"banned"})
	now := nowUTC
	defer func() { nowUTC = now }()
	nowUTC = func() time.Time { return time.Unix(0, 0).UTC() }

	approved := policy.Evaluate(Job{Body: "hello world"})
	if approved.Decision != DecisionApproved {
		t.Fatalf("expected approved, got %s", approved.Decision)
	}

	flagged := policy.Evaluate(Job{Body: "this contains BANNED content"})
	if flagged.Decision != DecisionFlagged {
		t.Fatalf("expected flagged, got %s", flagged.Decision)
	}
	if flagged.Reason == "" {
		t.Fatal("expected reason to be populated")
	}
}
