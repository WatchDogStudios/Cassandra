package config

import (
	"os"
	"strconv"
	"time"
)

// Loader provides convenient helpers for reading configuration values
// scoped by a common environment variable prefix (e.g. METRICS_, LOGS_).
type Loader struct {
	Prefix string
}

// NewLoader constructs a loader with the provided prefix. The prefix is
// automatically suffixed with an underscore when reading variables.
func NewLoader(prefix string) Loader {
	if prefix != "" && !hasTrailingUnderscore(prefix) {
		prefix += "_"
	}
	return Loader{Prefix: prefix}
}

func hasTrailingUnderscore(s string) bool {
	return len(s) > 0 && s[len(s)-1] == '_'
}

// String returns the environment variable value or the provided default.
func (l Loader) String(key, def string) string {
	if val := os.Getenv(l.Prefix + key); val != "" {
		return val
	}
	return def
}

// Int returns an integer environment variable or the provided default.
func (l Loader) Int(key string, def int) int {
	if val := os.Getenv(l.Prefix + key); val != "" {
		if parsed, err := strconv.Atoi(val); err == nil {
			return parsed
		}
	}
	return def
}

// Duration returns a duration environment variable (in seconds) or the default.
func (l Loader) Duration(key string, def time.Duration) time.Duration {
	if val := os.Getenv(l.Prefix + key); val != "" {
		if parsed, err := strconv.ParseFloat(val, 64); err == nil {
			return time.Duration(parsed * float64(time.Second))
		}
	}
	return def
}

// Bool returns a boolean environment variable or the default.
func (l Loader) Bool(key string, def bool) bool {
	if val := os.Getenv(l.Prefix + key); val != "" {
		if parsed, err := strconv.ParseBool(val); err == nil {
			return parsed
		}
	}
	return def
}
