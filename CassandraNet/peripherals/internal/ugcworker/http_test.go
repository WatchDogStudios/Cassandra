package ugcworker

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestServiceWorkflow(t *testing.T) {
	pool := NewWorkerPool(1, 4, NewModerationPolicy([]string{"ban"}), silentLogger{})
	pool.Start()

	svc := NewService(pool, silentLogger{})
	server := httptest.NewServer(svc.Handler())
	defer server.Close()
	defer func() {
		pool.Stop()
		svc.Shutdown()
	}()

	payload := map[string]string{
		"content_id": "42",
		"author_id":  "user",
		"body":       "contains ban term",
	}
	body, _ := json.Marshal(payload)
	resp, err := http.Post(server.URL+"/jobs", "application/json", bytes.NewReader(body))
	if err != nil {
		t.Fatalf("enqueue request failed: %v", err)
	}
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("expected 202 got %d", resp.StatusCode)
	}
	_ = resp.Body.Close()

	var result Result
	deadline := time.Now().Add(time.Second)
	for {
		if time.Now().After(deadline) {
			t.Fatal("timed out waiting for result")
		}
		r, err := http.Get(server.URL + "/jobs/next")
		if err != nil {
			t.Fatalf("next request failed: %v", err)
		}
		if r.StatusCode == http.StatusNoContent {
			_ = r.Body.Close()
			time.Sleep(20 * time.Millisecond)
			continue
		}
		if r.StatusCode != http.StatusOK {
			t.Fatalf("expected 200 or 204 got %d", r.StatusCode)
		}
		if err := json.NewDecoder(r.Body).Decode(&result); err != nil {
			t.Fatalf("decode failed: %v", err)
		}
		_ = r.Body.Close()
		break
	}

	if result.Decision != DecisionFlagged {
		t.Fatalf("expected flagged decision, got %s", result.Decision)
	}
}
