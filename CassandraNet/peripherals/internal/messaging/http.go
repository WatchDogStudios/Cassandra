package messaging

import (
	"encoding/json"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"
)

const topicsPrefix = "/topics/"

// Handler returns the HTTP handler for messaging endpoints.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	mux.HandleFunc(topicsPrefix, s.handleTopicRoute)
	return mux
}

type publishPayload struct {
	TenantID      string            `json:"tenant_id"`
	ProjectID     string            `json:"project_id"`
	Key           string            `json:"key"`
	PayloadBase64 string            `json:"payload_base64"`
	Priority      string            `json:"priority"`
	Attributes    map[string]string `json:"attributes"`
}

type messageResponse struct {
	MessageID     string            `json:"message_id"`
	TenantID      string            `json:"tenant_id"`
	ProjectID     string            `json:"project_id"`
	Topic         string            `json:"topic"`
	Key           string            `json:"key"`
	Priority      string            `json:"priority"`
	PublishedAt   string            `json:"published_at"`
	Attributes    map[string]string `json:"attributes,omitempty"`
	PayloadBase64 string            `json:"payload_base64"`
}

func (s *Service) handleTopicRoute(w http.ResponseWriter, r *http.Request) {
	if !strings.HasPrefix(r.URL.Path, topicsPrefix) {
		http.NotFound(w, r)
		return
	}
	rest := strings.TrimPrefix(r.URL.Path, topicsPrefix)
	segments := strings.Split(rest, "/")
	if len(segments) < 2 {
		http.NotFound(w, r)
		return
	}
	topic := segments[0]
	if topic == "" {
		http.NotFound(w, r)
		return
	}

	switch {
	case len(segments) == 2 && segments[1] == "messages":
		s.handleTopicMessages(w, r, topic)
	case len(segments) == 4 && segments[1] == "messages" && segments[3] == "ack":
		s.handleAck(w, r, topic, segments[2])
	default:
		http.NotFound(w, r)
	}
}

func (s *Service) handleTopicMessages(w http.ResponseWriter, r *http.Request, topic string) {
	switch r.Method {
	case http.MethodPost:
		s.handlePublish(w, r, topic)
	case http.MethodGet:
		s.handlePull(w, r, topic)
	default:
		headerAllow(w, http.MethodPost, http.MethodGet)
	}
}

func (s *Service) handlePublish(w http.ResponseWriter, r *http.Request, topic string) {
	defer r.Body.Close()
	var payload publishPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "invalid json payload", http.StatusBadRequest)
		return
	}
	bytes, err := DecodePayloadBase64(payload.PayloadBase64)
	if err != nil {
		http.Error(w, "invalid base64 payload", http.StatusBadRequest)
		return
	}
	priority := Priority(payload.Priority)
	if payload.Priority != "" {
		parsed, err := ParsePriority(payload.Priority)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		priority = parsed
	}
	message, err := s.Publish(r.Context(), PublishRequest{
		TenantID:   payload.TenantID,
		ProjectID:  payload.ProjectID,
		Topic:      topic,
		Key:        payload.Key,
		Payload:    bytes,
		Priority:   priority,
		Attributes: payload.Attributes,
	})
	if err != nil {
		httpError(w, err)
		return
	}
	resp := toMessageResponse(message)
	writeJSON(w, http.StatusCreated, resp)
}

func (s *Service) handlePull(w http.ResponseWriter, r *http.Request, topic string) {
	filter := PullFilter{
		TenantID:  r.URL.Query().Get("tenant_id"),
		ProjectID: r.URL.Query().Get("project_id"),
		Topic:     topic,
	}
	if limitStr := r.URL.Query().Get("limit"); limitStr != "" {
		if parsed, err := strconv.Atoi(limitStr); err == nil {
			filter.Limit = parsed
		}
	}
	messages, err := s.Pull(r.Context(), filter)
	if err != nil {
		httpError(w, err)
		return
	}
	resp := make([]messageResponse, 0, len(messages))
	for _, message := range messages {
		resp = append(resp, toMessageResponse(message))
	}
	writeJSON(w, http.StatusOK, resp)
}

func (s *Service) handleAck(w http.ResponseWriter, r *http.Request, topic, messageID string) {
	if r.Method != http.MethodPost {
		headerAllow(w, http.MethodPost)
		return
	}
	if err := s.Ack(r.Context(), topic, messageID); err != nil {
		httpError(w, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func toMessageResponse(message Message) messageResponse {
	return messageResponse{
		MessageID:     message.MessageID,
		TenantID:      message.TenantID,
		ProjectID:     message.ProjectID,
		Topic:         message.Topic,
		Key:           message.Key,
		Priority:      string(message.Priority),
		PublishedAt:   message.PublishedAt.UTC().Format(time.RFC3339Nano),
		Attributes:    cloneMap(message.Attributes),
		PayloadBase64: EncodePayloadBase64(message),
	}
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func httpError(w http.ResponseWriter, err error) {
	if errors.Is(err, ErrMessageNotFound) {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}
	http.Error(w, err.Error(), http.StatusBadRequest)
}

func headerAllow(w http.ResponseWriter, methods ...string) {
	w.Header().Set("Allow", strings.Join(methods, ", "))
	http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
}
