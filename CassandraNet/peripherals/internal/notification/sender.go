package notification

import "sync"

// Sender delivers a rendered notification body to a recipient.
type Sender interface {
	Send(Delivery) error
}

// MemorySender stores deliveries in memory for inspection/testing.
type MemorySender struct {
	mu         sync.Mutex
	deliveries []Delivery
}

// NewMemorySender constructs an empty memory sender.
func NewMemorySender() *MemorySender {
	return &MemorySender{}
}

// Send records the delivery.
func (m *MemorySender) Send(delivery Delivery) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.deliveries = append(m.deliveries, delivery)
	return nil
}

// Deliveries returns a copy of deliveries seen so far.
func (m *MemorySender) Deliveries() []Delivery {
	m.mu.Lock()
	defer m.mu.Unlock()
	out := make([]Delivery, len(m.deliveries))
	copy(out, m.deliveries)
	return out
}
