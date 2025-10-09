package notification

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// Service exposes HTTP endpoints for dispatching notifications.
type Service struct {
	templates *TemplateStore
	senders   map[Channel]Sender
	history   *History
	logger    interface {
		Printf(string, ...any)
	}
}

// NewService constructs a Service instance.
func NewService(templates *TemplateStore, senders map[Channel]Sender, history *History, logger interface {
	Printf(string, ...any)
}) *Service {
	return &Service{
		templates: templates,
		senders:   senders,
		history:   history,
		logger:    logger,
	}
}

// Handler returns the HTTP handler.
func (s *Service) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", s.handleHealth)
	mux.HandleFunc("/notify", s.handleNotify)
	mux.HandleFunc("/notifications/recent", s.handleRecent)
	return mux
}

func (s *Service) handleHealth(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("ok"))
}

func (s *Service) handleNotify(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	defer r.Body.Close()

	var msg Message
	if err := json.NewDecoder(r.Body).Decode(&msg); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}
	if msg.Channel == "" || msg.Recipient == "" || msg.Template == "" {
		http.Error(w, "channel, recipient, and template required", http.StatusBadRequest)
		return
	}

	sender, ok := s.senders[msg.Channel]
	if !ok {
		http.Error(w, fmt.Sprintf("unsupported channel %s", msg.Channel), http.StatusBadRequest)
		return
	}

	body, err := s.templates.Render(msg.Template, msg.Data)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	delivery := Delivery{
		Channel:   msg.Channel,
		Recipient: msg.Recipient,
		Body:      body,
		SentAt:    time.Now().UTC(),
	}
	if err := sender.Send(delivery); err != nil {
		http.Error(w, "failed to dispatch notification", http.StatusInternalServerError)
		return
	}
	s.history.Add(delivery)
	s.logger.Printf("sent %s notification to %s via template %s", msg.Channel, msg.Recipient, msg.Template)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(delivery)
}

func (s *Service) handleRecent(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	recent := s.history.Recent()
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(recent)
}
