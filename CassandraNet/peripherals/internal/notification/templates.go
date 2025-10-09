package notification

import (
	"fmt"
	"strings"
	"sync"
	"text/template"
)

// TemplateStore compiles and renders named templates for notifications.
type TemplateStore struct {
	mu        sync.RWMutex
	templates map[string]*template.Template
	raw       map[string]string
}

// NewTemplateStore seeds the store with basic templates.
func NewTemplateStore() *TemplateStore {
	store := &TemplateStore{
		templates: make(map[string]*template.Template),
		raw:       make(map[string]string),
	}
	// default templates
	_ = store.Register("welcome_email", "Hello {{.Name}}, welcome to CassandraNet!")
	_ = store.Register("password_reset", "Hi {{.Name}}, use code {{.Code}} to reset your password.")
	_ = store.Register("moderation_alert", "Content {{.ContentID}} was flagged for review.")
	return store
}

// Register adds or replaces a template definition.
func (s *TemplateStore) Register(name, body string) error {
	tmpl, err := template.New(name).Parse(body)
	if err != nil {
		return fmt.Errorf("parse template %s: %w", name, err)
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	s.templates[name] = tmpl
	s.raw[name] = body
	return nil
}

// Render executes the template with the provided data.
func (s *TemplateStore) Render(name string, data any) (string, error) {
	s.mu.RLock()
	tmpl, ok := s.templates[name]
	s.mu.RUnlock()
	if !ok {
		return "", fmt.Errorf("template %s not found", name)
	}
	var out strings.Builder
	if err := tmpl.Execute(&out, data); err != nil {
		return "", fmt.Errorf("render template %s: %w", name, err)
	}
	return out.String(), nil
}

// Raw returns the raw template text if present.
func (s *TemplateStore) Raw(name string) (string, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	body, ok := s.raw[name]
	return body, ok
}
