package server

import (
	"context"
	"net/http"
	"time"
)

// Run starts the HTTP server and blocks until the provided context is cancelled.
// It performs a graceful shutdown with a configurable timeout.
func Run(ctx context.Context, srv *http.Server, shutdownTimeout time.Duration) error {
	errCh := make(chan error, 1)
	go func() {
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
		close(errCh)
	}()

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
		defer cancel()
		return srv.Shutdown(shutdownCtx)
	case err := <-errCh:
		return err
	}
}
