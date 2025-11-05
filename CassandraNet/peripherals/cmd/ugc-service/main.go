package main

import (
	"context"
	"net/http"
	"os/signal"
	"syscall"
	"time"

	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/config"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logging"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/server"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/ugc"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	loader := config.NewLoader("UGC_SERVICE")
	addr := loader.String("HTTP_ADDR", ":8091")

	logger := logging.New("ugc-service")
	store := ugc.NewMemoryStore()
	svc := ugc.NewService(store, nil)

	srv := &http.Server{
		Addr:    addr,
		Handler: svc.Handler(),
	}

	logger.Printf("ugc service listening on %s", addr)
	if err := server.Run(ctx, srv, 5*time.Second); err != nil {
		logger.Printf("server shutdown: %v", err)
	}
}
