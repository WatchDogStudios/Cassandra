package metricscollector

import (
	"testing"
	"time"
)

func TestAggregatorIngestAndSnapshot(t *testing.T) {
	agg := NewAggregator()
	now := time.Now()

	agg.Ingest(MetricEvent{
		Namespace: "api",
		Name:      "latency",
		Value:     120,
		Labels: map[string]string{
			"method": "GET",
			"route":  "/foo",
		},
		Timestamp: now,
	})

	agg.Ingest(MetricEvent{
		Namespace: "api",
		Name:      "latency",
		Value:     80,
		Labels: map[string]string{
			"route":  "/foo",
			"method": "GET",
		},
		Timestamp: now.Add(time.Second),
	})

	snapshot := agg.Snapshot()
	if len(snapshot) != 1 {
		t.Fatalf("expected 1 metric summary, got %d", len(snapshot))
	}

	summary := snapshot["api.latency{method=GET,route=/foo}"]
	if summary.Count != 2 {
		t.Fatalf("expected count=2 got %d", summary.Count)
	}
	if summary.Min != 80 || summary.Max != 120 {
		t.Fatalf("unexpected min/max: %+v", summary)
	}
	if summary.Mean != 100 {
		t.Fatalf("expected mean=100 got %.2f", summary.Mean)
	}
	if summary.Last.IsZero() {
		t.Fatal("expected last timestamp to be set")
	}
}
