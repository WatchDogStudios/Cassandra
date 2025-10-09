package logpipeline

import "sync"

// RingBufferSink keeps the most recent log events in memory for debugging.
type RingBufferSink struct {
	mu       sync.RWMutex
	capacity int
	entries  []LogEvent
}

// NewRingBufferSink constructs a sink with bounded capacity.
func NewRingBufferSink(capacity int) *RingBufferSink {
	if capacity <= 0 {
		capacity = 100
	}
	return &RingBufferSink{capacity: capacity}
}

// Consume stores the event, evicting the oldest when capacity is exceeded.
func (r *RingBufferSink) Consume(event LogEvent) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, event)
	if len(r.entries) > r.capacity {
		r.entries = r.entries[len(r.entries)-r.capacity:]
	}
	return nil
}

// Recent returns a snapshot of the buffered events in chronological order.
func (r *RingBufferSink) Recent() []LogEvent {
	r.mu.RLock()
	defer r.mu.RUnlock()
	snapshot := make([]LogEvent, len(r.entries))
	copy(snapshot, r.entries)
	return snapshot
}
