package stream

import (
	"context"

	"github.com/siddontang/go-mysql/replication"
)

// A Position is a source-defined location from which to start streaming.
type Position interface{}

// An EventStream is an abstraction over an active stream of events.
type EventStream interface {
	// NextEvent returns the next event in the stream.
	NextEvent(ctx context.Context) (Event, error)
}

// An Event represents a single event that occurred in the source database.
type Event replication.Event
