package main

import (
	"context"
	"net/http"
	"os/signal"
	"syscall"
	"time"

	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/config"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logging"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/metricscollector"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/server"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	loader := config.NewLoader("METRICS")
	addr := loader.String("HTTP_ADDR", ":8081")

	logger := logging.New("metrics-collector")
	aggregator := metricscollector.NewAggregator()
	svc := metricscollector.NewService(aggregator, logger)

	srv := &http.Server{
		Addr:    addr,
		Handler: svc.Handler(),
	}

	logger.Printf("listening on %s", addr)
	if err := server.Run(ctx, srv, 5*time.Second); err != nil {
		logger.Printf("server shutdown: %v", err)
	}
}
