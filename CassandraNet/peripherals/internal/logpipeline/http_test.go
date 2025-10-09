package logpipeline

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestServiceIngestAndRecent(t *testing.T) {
	logger := noOpLogger{}
	pipeline := NewPipeline(4, LevelDebug, logger)
	ring := NewRingBufferSink(10)
	pipeline.RegisterSink(ring)
	pipeline.Start()
	defer pipeline.Stop()

	svc := NewService(pipeline, ring, logger)
	server := httptest.NewServer(svc.Handler())
	t.Cleanup(server.Close)

	payload := map[string]any{
		"source":  "svc",
		"level":   "info",
		"message": "hello world",
		"fields": map[string]string{
			"request_id": "abc",
		},
	}
	body, _ := json.Marshal(payload)
	resp, err := http.Post(server.URL+"/logs", "application/json", bytes.NewReader(body))
	if err != nil {
		t.Fatalf("ingest failed: %v", err)
	}
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("expected 202 got %d", resp.StatusCode)
	}
	_ = resp.Body.Close()

	// wait for pipeline to process
	time.Sleep(50 * time.Millisecond)

	resp, err = http.Get(server.URL + "/logs/recent")
	if err != nil {
		t.Fatalf("recent failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 got %d", resp.StatusCode)
	}
	var recent []LogEvent
	if err := json.NewDecoder(resp.Body).Decode(&recent); err != nil {
		t.Fatalf("decode failed: %v", err)
	}
	_ = resp.Body.Close()

	if len(recent) == 0 {
		t.Fatal("expected at least one log event")
	}
}
