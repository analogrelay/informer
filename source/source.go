package source

import "github.com/anurse/informer/stream"

// An EventSource is an abstraction over a source server that can emit change events.
type EventSource interface {
	StartStream(position stream.Position) (stream.EventStream, error)
}
