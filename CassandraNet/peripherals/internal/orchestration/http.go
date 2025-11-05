package orchestration

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
)

const assignmentsPathPrefix = "/assignments/"

// Handler returns an http.Handler exposing orchestration endpoints.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc("/assignments", s.handleAssignments)
	mux.HandleFunc(assignmentsPathPrefix, s.handleAssignmentByID)
	return mux
}

type assignPayload struct {
	AgentID    string            `json:"agent_id"`
	WorkloadID string            `json:"workload_id"`
	TenantID   string            `json:"tenant_id"`
	ProjectID  string            `json:"project_id"`
	Metadata   map[string]string `json:"metadata"`
}

type updatePayload struct {
	Status        string `json:"status"`
	StatusMessage string `json:"status_message"`
}

func (s *Service) handleAssignments(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		s.handleAssign(w, r)
	case http.MethodGet:
		s.handleList(w, r)
	default:
		headerAllow(w, http.MethodPost, http.MethodGet)
	}
}

func (s *Service) handleAssign(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var payload assignPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json payload", http.StatusBadRequest)
		return
	}
	assignment, err := s.AssignWork(r.Context(), AssignRequest{
		AgentID:    payload.AgentID,
		WorkloadID: payload.WorkloadID,
		TenantID:   payload.TenantID,
		ProjectID:  payload.ProjectID,
		Metadata:   payload.Metadata,
	})
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusCreated, assignment)
}

func (s *Service) handleList(w http.ResponseWriter, r *http.Request) {
	filter := ListAssignmentsFilter{
		AgentID:   r.URL.Query().Get("agent_id"),
		TenantID:  r.URL.Query().Get("tenant_id"),
		ProjectID: r.URL.Query().Get("project_id"),
	}
	if status := r.URL.Query().Get("status"); status != "" {
		parsed, err := ParseStatus(status)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		filter.Status = parsed
	}
	assignments, err := s.ListAssignments(r.Context(), filter)
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, assignments)
}

func (s *Service) handleAssignmentByID(w http.ResponseWriter, r *http.Request) {
	if !strings.HasPrefix(r.URL.Path, assignmentsPathPrefix) {
		http.NotFound(w, r)
		return
	}
	id := strings.TrimPrefix(r.URL.Path, assignmentsPathPrefix)
	if id == "" {
		http.NotFound(w, r)
		return
	}
	switch r.Method {
	case http.MethodPatch:
		s.handleUpdate(w, r, id)
	default:
		headerAllow(w, http.MethodPatch)
	}
}

func (s *Service) handleUpdate(w http.ResponseWriter, r *http.Request, id string) {
	defer r.Body.Close()
	var payload updatePayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json payload", http.StatusBadRequest)
		return
	}
	status, err := ParseStatus(payload.Status)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	assignment, err := s.UpdateStatus(r.Context(), UpdateStatusRequest{
		AssignmentID:  id,
		Status:        status,
		StatusMessage: payload.StatusMessage,
	})
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, assignment)
}

// ParseStatus parses a string into a Status value.
func ParseStatus(status string) (Status, error) {
	switch strings.ToLower(status) {
	case string(StatusPending), "pending":
		return StatusPending, nil
	case string(StatusAssigned), "assigned":
		return StatusAssigned, nil
	case string(StatusRunning), "in_progress", "running":
		return StatusRunning, nil
	case string(StatusCompleted), "completed":
		return StatusCompleted, nil
	case string(StatusFailed), "failed":
		return StatusFailed, nil
	case string(StatusCancelled), "cancelled", "canceled":
		return StatusCancelled, nil
	default:
		return "", errors.New("unknown status")
	}
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func httpError(w http.ResponseWriter, err error) {
	if errors.Is(err, ErrAssignmentNotFound) {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}
	http.Error(w, err.Error(), http.StatusBadRequest)
}

func headerAllow(w http.ResponseWriter, methods ...string) {
	w.Header().Set("Allow", strings.Join(methods, ", "))
	http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
}
