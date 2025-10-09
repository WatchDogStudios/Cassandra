package notification

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

type noopLogger struct{}

func (noopLogger) Printf(string, ...any) {
	// no-op: suppress logging during tests
}

func TestServiceNotifyAndRecent(t *testing.T) {
	templates := NewTemplateStore()
	history := NewHistory(10)
	sender := NewMemorySender()
	svc := NewService(templates, map[Channel]Sender{
		ChannelEmail: sender,
	}, history, noopLogger{})

	server := httptest.NewServer(svc.Handler())
	defer server.Close()

	payload := Message{
		Channel:   ChannelEmail,
		Recipient: "user@example.com",
		Template:  "welcome_email",
		Data: map[string]any{
			"Name": "Grace",
		},
	}
	body, _ := json.Marshal(payload)
	resp, err := http.Post(server.URL+"/notify", "application/json", bytes.NewReader(body))
	if err != nil {
		t.Fatalf("notify request failed: %v", err)
	}
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("expected 202 got %d", resp.StatusCode)
	}
	var delivery Delivery
	if err := json.NewDecoder(resp.Body).Decode(&delivery); err != nil {
		t.Fatalf("decode failed: %v", err)
	}
	_ = resp.Body.Close()
	if delivery.Body == "" {
		t.Fatal("expected body to be populated")
	}

	resp, err = http.Get(server.URL + "/notifications/recent")
	if err != nil {
		t.Fatalf("recent request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 got %d", resp.StatusCode)
	}
	var recents []Delivery
	if err := json.NewDecoder(resp.Body).Decode(&recents); err != nil {
		t.Fatalf("decode failed: %v", err)
	}
	_ = resp.Body.Close()
	if len(recents) != 1 {
		t.Fatalf("expected 1 delivery, got %d", len(recents))
	}
}
