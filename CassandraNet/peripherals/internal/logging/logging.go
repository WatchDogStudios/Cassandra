package logging

import (
	"log"
	"os"
)

// New creates a standard library logger with a consistent prefix and flags.
func New(service string) *log.Logger {
	prefix := "[" + service + "] "
	return log.New(os.Stdout, prefix, log.LstdFlags|log.Lmicroseconds|log.LUTC)
}
