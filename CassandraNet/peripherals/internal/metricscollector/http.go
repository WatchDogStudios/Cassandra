package metricscollector

import (
	"encoding/json"
	"net/http"
	"time"
)

// Service wires HTTP handlers to the underlying aggregator.
type Service struct {
	agg    *Aggregator
	logger interface {
		Printf(string, ...any)
	}
}

// NewService constructs a metrics service using the provided logger.
func NewService(agg *Aggregator, logger interface {
	Printf(string, ...any)
}) *Service {
	return &Service{agg: agg, logger: logger}
}

// Handler returns the HTTP handler that exposes ingest and summary endpoints.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", s.handleHealth)
	mux.HandleFunc("/metrics/ingest", s.handleIngest)
	mux.HandleFunc("/metrics/summary", s.handleSummary)
	return mux
}

func (s *Service) handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("ok"))
}

func (s *Service) handleIngest(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	defer r.Body.Close()

	var payload MetricEvent
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}
	if payload.Namespace == "" || payload.Name == "" {
		http.Error(w, "namespace and name required", http.StatusBadRequest)
		return
	}
	if payload.Timestamp.IsZero() {
		payload.Timestamp = time.Now().UTC()
	}
	summary := s.agg.Ingest(payload)
	s.logger.Printf("ingested metric %s.%s value=%.2f", payload.Namespace, payload.Name, payload.Value)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(summary)
}

func (s *Service) handleSummary(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	snapshot := s.agg.Snapshot()
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(snapshot)
}
