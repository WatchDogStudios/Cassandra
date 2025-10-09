package ugcworker

import (
	"testing"
	"time"
)

type silentLogger struct{}

func (silentLogger) Printf(string, ...any) {}

func TestWorkerPoolProcessesJobs(t *testing.T) {
	policy := NewModerationPolicy([]string{"banned"})
	pool := NewWorkerPool(1, 2, policy, silentLogger{})
	pool.Start()
	defer pool.Stop()

	job := Job{ContentID: "1", AuthorID: "user", Body: "clean content"}
	if err := pool.Enqueue(job); err != nil {
		t.Fatalf("enqueue failed: %v", err)
	}

	select {
	case result := <-pool.Results():
		if result.Decision != DecisionApproved {
			t.Fatalf("expected approved decision, got %s", result.Decision)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for result")
	}
}

func TestWorkerPoolQueueFull(t *testing.T) {
	policy := NewModerationPolicy(nil)
	pool := NewWorkerPool(1, 1, policy, silentLogger{})
	pool.Start()
	defer pool.Stop()

	job := Job{ContentID: "1", AuthorID: "user", Body: "clean"}
	if err := pool.Enqueue(job); err != nil {
		t.Fatalf("enqueue failed: %v", err)
	}
	if err := pool.Enqueue(job); err != ErrQueueFull {
		t.Fatalf("expected ErrQueueFull, got %v", err)
	}
}
