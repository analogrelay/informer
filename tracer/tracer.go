package tracer

import (
	"github.com/anurse/informer/source"
	"github.com/anurse/informer/stream"
)

// A Tracer represents a connection to a data source that can emit Events
type Tracer struct {
	source source.EventSource
}

// NewTracer creates a new tracer for the provided source
func NewTracer(src source.EventSource, position stream.Position) Tracer {
	return Tracer{
		source: src,
	}
}
