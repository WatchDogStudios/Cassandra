package logpipeline

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"
)

// Service exposes HTTP endpoints for the log pipeline.
type Service struct {
	pipeline *Pipeline
	ring     *RingBufferSink
	logger   interface {
		Printf(string, ...any)
	}
}

// NewService constructs a Service.
func NewService(pipeline *Pipeline, ring *RingBufferSink, logger interface {
	Printf(string, ...any)
}) *Service {
	return &Service{pipeline: pipeline, ring: ring, logger: logger}
}

// Handler returns the HTTP handler for the service.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", s.handleHealth)
	mux.HandleFunc("/logs", s.handleIngest)
	mux.HandleFunc("/logs/recent", s.handleRecent)
	return mux
}

func (s *Service) handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("ok"))
}

type logPayload struct {
	Source    string            `json:"source"`
	Level     string            `json:"level"`
	Message   string            `json:"message"`
	Fields    map[string]string `json:"fields"`
	Timestamp time.Time         `json:"timestamp"`
}

func (s *Service) handleIngest(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	defer r.Body.Close()

	var payload logPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}
	if payload.Source == "" || payload.Message == "" {
		http.Error(w, "source and message required", http.StatusBadRequest)
		return
	}
	event := LogEvent{
		Source:    payload.Source,
		Level:     ParseLevel(payload.Level),
		LevelName: strings.ToUpper(payload.Level),
		Message:   payload.Message,
		Fields:    payload.Fields,
	}
	if payload.Timestamp.IsZero() {
		event.Timestamp = time.Now().UTC()
	} else {
		event.Timestamp = payload.Timestamp
	}
	if event.LevelName == "" {
		event.LevelName = event.Level.String()
	}

	if err := s.pipeline.Enqueue(event); err != nil {
		if errors.Is(err, ErrBackpressure) {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		http.Error(w, "failed to enqueue log", http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusAccepted)
}

func (s *Service) handleRecent(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	events := s.ring.Recent()
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(events)
}
