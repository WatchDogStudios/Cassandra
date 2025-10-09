package logpipeline

// StdoutSink writes log events to the configured logger.
type StdoutSink struct {
	logger interface {
		Printf(string, ...any)
	}
}

// NewStdoutSink returns a sink that prints events.
func NewStdoutSink(logger interface {
	Printf(string, ...any)
}) StdoutSink {
	return StdoutSink{logger: logger}
}

// Consume formats the log event and prints it.
func (s StdoutSink) Consume(event LogEvent) error {
	s.logger.Printf("%s [%s] %s fields=%v", event.Source, event.LevelName, event.Message, event.Fields)
	return nil
}

func (lvl Level) String() string {
	switch lvl {
	case LevelDebug:
		return "DEBUG"
	case LevelWarn:
		return "WARN"
	case LevelError:
		return "ERROR"
	default:
		return "INFO"
	}
}

// FormatLevel converts to canonical upper-case representation.
func FormatLevel(lvl Level) string {
	return lvl.String()
}
