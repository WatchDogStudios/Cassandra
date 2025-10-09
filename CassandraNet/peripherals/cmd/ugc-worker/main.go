package main

import (
	"context"
	"net/http"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/config"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logging"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/server"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/ugcworker"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	loader := config.NewLoader("UGC")
	addr := loader.String("HTTP_ADDR", ":8083")
	queueSize := loader.Int("QUEUE_SIZE", 256)
	workerCount := loader.Int("WORKERS", 4)
	banned := parseBanned(loader.String("BANNED_TERMS", "spam,scam"))

	logger := logging.New("ugc-worker")
	policy := ugcworker.NewModerationPolicy(banned)
	pool := ugcworker.NewWorkerPool(workerCount, queueSize, policy, logger)
	pool.Start()

	service := ugcworker.NewService(pool, logger)

	srv := &http.Server{
		Addr:    addr,
		Handler: service.Handler(),
	}

	logger.Printf("listening on %s", addr)
	if err := server.Run(ctx, srv, 5*time.Second); err != nil {
		logger.Printf("server shutdown: %v", err)
	}
	pool.Stop()
	service.Shutdown()
}

func parseBanned(raw string) []string {
	if raw == "" {
		return nil
	}
	parts := strings.Split(raw, ",")
	out := make([]string, 0, len(parts))
	for _, part := range parts {
		trimmed := strings.TrimSpace(part)
		if trimmed != "" {
			out = append(out, trimmed)
		}
	}
	return out
}
