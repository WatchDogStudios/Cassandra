package metricscollector

import (
	"sort"
	"strings"
	"sync"
	"time"
)

// MetricEvent represents an incoming metric sample.
type MetricEvent struct {
	Namespace string            `json:"namespace"`
	Name      string            `json:"name"`
	Value     float64           `json:"value"`
	Labels    map[string]string `json:"labels"`
	Timestamp time.Time         `json:"timestamp"`
}

// Summary captures roll-up statistics for a set of samples.
type Summary struct {
	Count int       `json:"count"`
	Min   float64   `json:"min"`
	Max   float64   `json:"max"`
	Sum   float64   `json:"sum"`
	Mean  float64   `json:"mean"`
	Last  time.Time `json:"last"`
}

// Aggregator ingest metrics and maintains summaries per namespace/name/label set.
type Aggregator struct {
	mu      sync.RWMutex
	metrics map[string]Summary
}

// NewAggregator returns a zeroed aggregator instance.
func NewAggregator() *Aggregator {
	return &Aggregator{metrics: make(map[string]Summary)}
}

// Ingest adds a new metric event, updating the corresponding summary.
func (a *Aggregator) Ingest(event MetricEvent) Summary {
	key := eventKey(event)
	a.mu.Lock()
	defer a.mu.Unlock()

	summary, ok := a.metrics[key]
	if !ok {
		summary = Summary{
			Min: event.Value,
			Max: event.Value,
		}
	}
	if event.Value < summary.Min {
		summary.Min = event.Value
	}
	if event.Value > summary.Max {
		summary.Max = event.Value
	}
	summary.Count++
	summary.Sum += event.Value
	summary.Mean = summary.Sum / float64(summary.Count)
	summary.Last = event.Timestamp
	a.metrics[key] = summary
	return summary
}

// Snapshot returns a copy of the current summaries keyed by metric
// identity string `namespace.name{labels}`.
func (a *Aggregator) Snapshot() map[string]Summary {
	a.mu.RLock()
	defer a.mu.RUnlock()

	clone := make(map[string]Summary, len(a.metrics))
	for k, v := range a.metrics {
		clone[k] = v
	}
	return clone
}

func eventKey(event MetricEvent) string {
	var b strings.Builder
	b.WriteString(event.Namespace)
	b.WriteString(".")
	b.WriteString(event.Name)
	b.WriteString("{")
	if len(event.Labels) > 0 {
		keys := make([]string, 0, len(event.Labels))
		for k := range event.Labels {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		for i, k := range keys {
			if i > 0 {
				b.WriteString(",")
			}
			b.WriteString(k)
			b.WriteString("=")
			b.WriteString(event.Labels[k])
		}
	}
	b.WriteString("}")
	return b.String()
}
