package ugc

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
)

const (
	contentBasePath   = "/content"
	contentByIDPrefix = "/content/"
)

// Handler returns an HTTP handler for UGC moderation endpoints.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc(contentBasePath, s.handleContent)
	mux.HandleFunc(contentByIDPrefix, s.handleContentByID)
	return mux
}

type submitPayload struct {
	ContentID  string            `json:"content_id"`
	TenantID   string            `json:"tenant_id"`
	ProjectID  string            `json:"project_id"`
	Filename   string            `json:"filename"`
	MimeType   string            `json:"mime_type"`
	SizeBytes  uint64            `json:"size_bytes"`
	Labels     map[string]string `json:"labels"`
	Attributes map[string]string `json:"attributes"`
}

type reviewPayload struct {
	State  string `json:"state"`
	Reason string `json:"reason"`
}

func (s *Service) handleContent(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		s.handleSubmit(w, r)
	case http.MethodGet:
		s.handleList(w, r)
	default:
		headerAllow(w, http.MethodPost, http.MethodGet)
	}
}

func (s *Service) handleSubmit(w http.ResponseWriter, r *http.Request) {
	defer r.Body.Close()
	var payload submitPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json payload", http.StatusBadRequest)
		return
	}
	content, err := s.SubmitContent(r.Context(), SubmitRequest{
		ContentID:  payload.ContentID,
		TenantID:   payload.TenantID,
		ProjectID:  payload.ProjectID,
		Filename:   payload.Filename,
		MimeType:   payload.MimeType,
		SizeBytes:  payload.SizeBytes,
		Labels:     payload.Labels,
		Attributes: payload.Attributes,
	})
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusCreated, content)
}

func (s *Service) handleList(w http.ResponseWriter, r *http.Request) {
	filter := ListFilter{
		TenantID:  r.URL.Query().Get("tenant_id"),
		ProjectID: r.URL.Query().Get("project_id"),
	}
	if state := r.URL.Query().Get("state"); state != "" {
		parsed, err := ParseState(state)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		filter.State = parsed
	}
	items, err := s.ListContent(r.Context(), filter)
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, items)
}

func (s *Service) handleContentByID(w http.ResponseWriter, r *http.Request) {
	if !strings.HasPrefix(r.URL.Path, contentByIDPrefix) {
		http.NotFound(w, r)
		return
	}
	id := strings.TrimPrefix(r.URL.Path, contentByIDPrefix)
	if id == "" {
		http.NotFound(w, r)
		return
	}
	if strings.HasSuffix(id, "/review") {
		contentID := strings.TrimSuffix(id, "/review")
		if contentID == "" || strings.Contains(contentID, "/") {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodPost {
			headerAllow(w, http.MethodPost)
			return
		}
		s.handleReview(w, r, contentID)
		return
	}
	http.NotFound(w, r)
}

func (s *Service) handleReview(w http.ResponseWriter, r *http.Request, id string) {
	defer r.Body.Close()
	var payload reviewPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json payload", http.StatusBadRequest)
		return
	}
	state, err := ParseState(payload.State)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	content, err := s.ReviewContent(r.Context(), ReviewRequest{
		ContentID: id,
		State:     state,
		Reason:    payload.Reason,
	})
	if err != nil {
		httpError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, content)
}

// ParseState converts string representations into a State value.
func ParseState(value string) (State, error) {
	switch strings.ToLower(value) {
	case string(StatePending), "pending":
		return StatePending, nil
	case string(StateApproved), "approved":
		return StateApproved, nil
	case string(StateRejected), "rejected":
		return StateRejected, nil
	case string(StateArchived), "archived":
		return StateArchived, nil
	default:
		return "", errors.New("unknown state")
	}
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func httpError(w http.ResponseWriter, err error) {
	if errors.Is(err, ErrContentNotFound) {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}
	http.Error(w, err.Error(), http.StatusBadRequest)
}

func headerAllow(w http.ResponseWriter, methods ...string) {
	w.Header().Set("Allow", strings.Join(methods, ", "))
	http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
}
