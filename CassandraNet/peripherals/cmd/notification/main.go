package main

import (
	"context"
	"net/http"
	"os/signal"
	"syscall"
	"time"

	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/config"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/logging"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/notification"
	"github.com/WatchDogStudios/CassandraNet/peripherals/internal/server"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	loader := config.NewLoader("NOTIFY")
	addr := loader.String("HTTP_ADDR", ":8084")
	recentCapacity := loader.Int("RECENT_CAPACITY", 200)

	logger := logging.New("notification-service")
	templates := notification.NewTemplateStore()
	history := notification.NewHistory(recentCapacity)

	senders := map[notification.Channel]notification.Sender{
		notification.ChannelEmail:   notification.NewMemorySender(),
		notification.ChannelWebhook: notification.NewMemorySender(),
		notification.ChannelInApp:   notification.NewMemorySender(),
	}

	svc := notification.NewService(templates, senders, history, logger)
	srv := &http.Server{
		Addr:    addr,
		Handler: svc.Handler(),
	}

	logger.Printf("listening on %s", addr)
	if err := server.Run(ctx, srv, 5*time.Second); err != nil {
		logger.Printf("server shutdown: %v", err)
	}
}
