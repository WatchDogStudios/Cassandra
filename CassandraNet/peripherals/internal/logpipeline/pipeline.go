package logpipeline

import (
	"errors"
	"strings"
	"sync"
	"time"
)

var (
	// ErrBackpressure is returned when the pipeline queue is full.
	ErrBackpressure = errors.New("log pipeline backpressure: queue full")
)

// Level models a log severity.
type Level int

const (
	LevelDebug Level = iota
	LevelInfo
	LevelWarn
	LevelError
)

// ParseLevel converts a string to a Level, defaulting to INFO.
func ParseLevel(v string) Level {
	switch strings.ToUpper(v) {
	case "DEBUG":
		return LevelDebug
	case "WARN":
		return LevelWarn
	case "ERROR":
		return LevelError
	default:
		return LevelInfo
	}
}

// LogEvent is the payload for the log pipeline.
type LogEvent struct {
	Source    string            `json:"source"`
	Level     Level             `json:"-"`
	LevelName string            `json:"level"`
	Message   string            `json:"message"`
	Fields    map[string]string `json:"fields"`
	Timestamp time.Time         `json:"timestamp"`
}

// Sink receives processed log events.
type Sink interface {
	Consume(LogEvent) error
}

// Pipeline delivers log events to registered sinks with basic filtering.
type Pipeline struct {
	logger interface {
		Printf(string, ...any)
	}
	sinks    []Sink
	events   chan LogEvent
	minLevel Level
	wg       sync.WaitGroup
	once     sync.Once
	stopOnce sync.Once
}

// NewPipeline creates a pipeline with the specified buffer and minimum level.
func NewPipeline(buffer int, minLevel Level, logger interface {
	Printf(string, ...any)
}) *Pipeline {
	if buffer <= 0 {
		buffer = 64
	}
	p := &Pipeline{
		logger:   logger,
		events:   make(chan LogEvent, buffer),
		minLevel: minLevel,
	}
	return p
}

// RegisterSink registers a sink for processed events. It must be called before Start.
func (p *Pipeline) RegisterSink(s Sink) {
	p.sinks = append(p.sinks, s)
}

// Start launches the dispatch loop.
func (p *Pipeline) Start() {
	p.once.Do(func() {
		p.wg.Add(1)
		go func() {
			defer p.wg.Done()
			for event := range p.events {
				for _, sink := range p.sinks {
					if err := sink.Consume(event); err != nil {
						p.logger.Printf("log sink error: %v", err)
					}
				}
			}
		}()
	})
}

// Stop waits for the dispatch loop to drain remaining events.
func (p *Pipeline) Stop() {
	p.stopOnce.Do(func() {
		close(p.events)
		p.wg.Wait()
	})
}

// Enqueue submits a log event for processing.
func (p *Pipeline) Enqueue(event LogEvent) error {
	if event.Level < p.minLevel {
		return nil
	}
	select {
	case p.events <- event:
		return nil
	default:
		return ErrBackpressure
	}
}
