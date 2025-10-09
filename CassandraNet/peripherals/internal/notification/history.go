package notification

import "sync"

// History keeps a bounded list of recent deliveries.
type History struct {
	mu       sync.RWMutex
	capacity int
	entries  []Delivery
}

// NewHistory constructs a history with the provided capacity.
func NewHistory(capacity int) *History {
	if capacity <= 0 {
		capacity = 100
	}
	return &History{capacity: capacity}
}

// Add records a delivery.
func (h *History) Add(delivery Delivery) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.entries = append(h.entries, delivery)
	if len(h.entries) > h.capacity {
		h.entries = h.entries[len(h.entries)-h.capacity:]
	}
}

// Recent returns the stored deliveries in chronological order.
func (h *History) Recent() []Delivery {
	h.mu.RLock()
	defer h.mu.RUnlock()
	snapshot := make([]Delivery, len(h.entries))
	copy(snapshot, h.entries)
	return snapshot
}
