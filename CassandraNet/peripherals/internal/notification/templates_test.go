package notification

import "testing"

func TestTemplateStoreRender(t *testing.T) {
	store := NewTemplateStore()
	if err := store.Register("custom", "Hello {{.Name}}"); err != nil {
		t.Fatalf("register failed: %v", err)
	}
	body, err := store.Render("custom", map[string]string{"Name": "Ada"})
	if err != nil {
		t.Fatalf("render failed: %v", err)
	}
	if body != "Hello Ada" {
		t.Fatalf("unexpected body: %s", body)
	}
}
