package metricscollector

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

type testLogger struct{}

// Printf implements the logger interface but discards output for tests.
func (testLogger) Printf(string, ...any) {
	// no-op: test logger suppresses output
}

func TestServiceIngestAndSummary(t *testing.T) {
	agg := NewAggregator()
	svc := NewService(agg, testLogger{})
	server := httptest.NewServer(svc.Handler())
	t.Cleanup(server.Close)

	payload := MetricEvent{
		Namespace: "core",
		Name:      "requests",
		Value:     1,
		Labels: map[string]string{
			"status": "200",
		},
		Timestamp: time.Now(),
	}
	body, _ := json.Marshal(payload)
	resp, err := http.Post(server.URL+"/metrics/ingest", "application/json", bytes.NewReader(body))
	if err != nil {
		t.Fatalf("ingest request failed: %v", err)
	}
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("expected 202 got %d", resp.StatusCode)
	}
	_ = resp.Body.Close()

	resp, err = http.Get(server.URL + "/metrics/summary")
	if err != nil {
		t.Fatalf("summary request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 got %d", resp.StatusCode)
	}
	var snapshot map[string]Summary
	if err := json.NewDecoder(resp.Body).Decode(&snapshot); err != nil {
		t.Fatalf("failed to decode summary: %v", err)
	}
	_ = resp.Body.Close()

	if len(snapshot) != 1 {
		t.Fatalf("expected 1 summary, got %d", len(snapshot))
	}
}
