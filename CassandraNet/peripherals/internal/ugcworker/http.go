package ugcworker

import (
	"encoding/json"
	"errors"
	"net/http"
	"sync"
	"time"
)

// Service exposes HTTP endpoints for managing UGC moderation jobs.
type Service struct {
	pool    *WorkerPool
	results *resultStore
	logger  interface {
		Printf(string, ...any)
	}
	collectorWg sync.WaitGroup
}

// NewService constructs a Service and starts the result collector loop.
func NewService(pool *WorkerPool, logger interface {
	Printf(string, ...any)
}) *Service {
	svc := &Service{
		pool:    pool,
		results: &resultStore{},
		logger:  logger,
	}
	svc.collectorWg.Add(1)
	go svc.collectResults()
	return svc
}

func (s *Service) collectResults() {
	defer s.collectorWg.Done()
	for result := range s.pool.Results() {
		s.results.push(result)
	}
}

// Shutdown waits for the result collector to finish.
func (s *Service) Shutdown() {
	s.collectorWg.Wait()
}

// Handler returns the HTTP handler.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", s.handleHealth)
	mux.HandleFunc("/jobs", s.handleEnqueue)
	mux.HandleFunc("/jobs/next", s.handleNext)
	return mux
}

func (s *Service) handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("ok"))
}

type enqueuePayload struct {
	ContentID string `json:"content_id"`
	AuthorID  string `json:"author_id"`
	Body      string `json:"body"`
}

func (s *Service) handleEnqueue(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	defer r.Body.Close()

	var payload enqueuePayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}
	if payload.ContentID == "" || payload.AuthorID == "" || payload.Body == "" {
		http.Error(w, "content_id, author_id, and body required", http.StatusBadRequest)
		return
	}
	job := Job{
		ContentID: payload.ContentID,
		AuthorID:  payload.AuthorID,
		Body:      payload.Body,
		Submitted: time.Now().UTC(),
	}
	if err := s.pool.Enqueue(job); err != nil {
		if errors.Is(err, ErrQueueFull) {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		http.Error(w, "failed to enqueue job", http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusAccepted)
}

func (s *Service) handleNext(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	if result, ok := s.results.pop(); ok {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(result)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

type resultStore struct {
	mu     sync.Mutex
	queued []Result
}

func (r *resultStore) push(result Result) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.queued = append(r.queued, result)
}

func (r *resultStore) pop() (Result, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if len(r.queued) == 0 {
		return Result{}, false
	}
	result := r.queued[0]
	r.queued = r.queued[1:]
	return result, true
}
