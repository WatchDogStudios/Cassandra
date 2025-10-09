package logpipeline

import (
	"errors"
	"sync"
	"testing"
	"time"
)

type captureSink struct {
	mu     sync.Mutex
	events []LogEvent
}

func (c *captureSink) Consume(event LogEvent) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.events = append(c.events, event)
	return nil
}

func (c *captureSink) snapshot() []LogEvent {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]LogEvent, len(c.events))
	copy(out, c.events)
	return out
}

type noOpLogger struct{}

func (noOpLogger) Printf(string, ...any) {}

func TestPipelineEnqueueAndDispatch(t *testing.T) {
	logger := noOpLogger{}
	pipeline := NewPipeline(2, LevelInfo, logger)
	sink := &captureSink{}
	pipeline.RegisterSink(sink)
	pipeline.Start()
	defer pipeline.Stop()

	evt := LogEvent{Source: "svc", Level: LevelInfo, LevelName: "INFO", Message: "hello", Timestamp: time.Now()}
	if err := pipeline.Enqueue(evt); err != nil {
		t.Fatalf("enqueue failed: %v", err)
	}

	time.Sleep(50 * time.Millisecond)
	events := sink.snapshot()
	if len(events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(events))
	}
}

func TestPipelineBackpressure(t *testing.T) {
	logger := noOpLogger{}
	pipeline := NewPipeline(1, LevelInfo, logger)
	sink := &captureSink{}
	pipeline.RegisterSink(sink)
	pipeline.Start()
	defer pipeline.Stop()

	evt := LogEvent{Source: "svc", Level: LevelInfo, LevelName: "INFO", Message: "hello", Timestamp: time.Now()}
	if err := pipeline.Enqueue(evt); err != nil {
		t.Fatalf("enqueue failed: %v", err)
	}
	if err := pipeline.Enqueue(evt); !errors.Is(err, ErrBackpressure) {
		t.Fatalf("expected backpressure error, got %v", err)
	}
}

func TestRingBufferCapacity(t *testing.T) {
	ring := NewRingBufferSink(2)
	_ = ring.Consume(LogEvent{Message: "first"})
	_ = ring.Consume(LogEvent{Message: "second"})
	_ = ring.Consume(LogEvent{Message: "third"})

	recent := ring.Recent()
	if len(recent) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(recent))
	}
	if recent[0].Message != "second" || recent[1].Message != "third" {
		t.Fatalf("unexpected order: %+v", recent)
	}
}
