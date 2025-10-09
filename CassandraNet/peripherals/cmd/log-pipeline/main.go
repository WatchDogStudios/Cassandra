package main

import (
	"context"
	"net/http"
	"os/signal"
	"syscall"
	"time"

	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/config"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logging"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logpipeline"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/server"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	loader := config.NewLoader("LOG_PIPELINE")
	addr := loader.String("HTTP_ADDR", ":8082")
	buffer := loader.Int("QUEUE_SIZE", 256)
	minLevel := logpipeline.ParseLevel(loader.String("MIN_LEVEL", "INFO"))
	recentCapacity := loader.Int("RECENT_CAPACITY", 200)

	logger := logging.New("log-pipeline")
	pipeline := logpipeline.NewPipeline(buffer, minLevel, logger)
	ring := logpipeline.NewRingBufferSink(recentCapacity)
	pipeline.RegisterSink(ring)
	pipeline.RegisterSink(logpipeline.NewStdoutSink(logger))
	pipeline.Start()
	defer pipeline.Stop()

	svc := logpipeline.NewService(pipeline, ring, logger)
	srv := &http.Server{
		Addr:    addr,
		Handler: svc.Handler(),
	}

	logger.Printf("listening on %s", addr)
	if err := server.Run(ctx, srv, 5*time.Second); err != nil {
		logger.Printf("server shutdown: %v", err)
	}
}
