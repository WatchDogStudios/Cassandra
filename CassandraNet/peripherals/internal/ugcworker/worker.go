package ugcworker

import (
	"errors"
	"sync"
	"time"
)

var (
	// ErrQueueFull indicates the job queue is currently saturated.
	ErrQueueFull = errors.New("ugc queue full")
)

// WorkerPool processes moderation jobs concurrently.
type WorkerPool struct {
	policy  ModerationPolicy
	jobs    chan Job
	results chan Result
	workers int
	logger  interface {
		Printf(string, ...any)
	}
	startOnce sync.Once
	stopOnce  sync.Once
	wg        sync.WaitGroup
}

// NewWorkerPool constructs a worker pool.
func NewWorkerPool(workers, queueSize int, policy ModerationPolicy, logger interface {
	Printf(string, ...any)
}) *WorkerPool {
	if workers <= 0 {
		workers = 2
	}
	if queueSize <= 0 {
		queueSize = 128
	}
	return &WorkerPool{
		policy:  policy,
		jobs:    make(chan Job, queueSize),
		results: make(chan Result, queueSize),
		workers: workers,
		logger:  logger,
	}
}

// Start launches worker goroutines.
func (p *WorkerPool) Start() {
	p.startOnce.Do(func() {
		for i := 0; i < p.workers; i++ {
			p.wg.Add(1)
			go p.workerLoop()
		}
	})
}

func (p *WorkerPool) workerLoop() {
	defer p.wg.Done()
	for job := range p.jobs {
		if job.Submitted.IsZero() {
			job.Submitted = time.Now().UTC()
		}
		result := p.policy.Evaluate(job)
		select {
		case p.results <- result:
		default:
			p.logger.Printf("dropping UGC result for %s: results channel full", job.ContentID)
		}
	}
}

// Stop drains workers and closes the results channel.
func (p *WorkerPool) Stop() {
	p.stopOnce.Do(func() {
		close(p.jobs)
		p.wg.Wait()
		close(p.results)
	})
}

// Enqueue submits a job for moderation.
func (p *WorkerPool) Enqueue(job Job) error {
	select {
	case p.jobs <- job:
		return nil
	default:
		return ErrQueueFull
	}
}

// Results exposes a read-only channel of moderation results.
func (p *WorkerPool) Results() <-chan Result {
	return p.results
}
