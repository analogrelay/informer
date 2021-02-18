package mysql

import (
	"context"
	"fmt"
	"reflect"

	"github.com/anurse/informer/source"
	"github.com/anurse/informer/stream"
	"github.com/siddontang/go-log/log"
	"github.com/siddontang/go-mysql/client"
	"github.com/siddontang/go-mysql/mysql"
	go_mysql "github.com/siddontang/go-mysql/mysql"
	"github.com/siddontang/go-mysql/replication"
	"github.com/siddontang/go-mysql/schema"
)

var _ stream.EventStream = &mySQLStream{}
var _ source.EventSource = &Source{}

type mySQLPosition struct {
	hasGtid bool
	binlog  go_mysql.Position
	gtidSet go_mysql.GTIDSet
}

func (p *mySQLPosition) String() string {
	return fmt.Sprintf("%s:%d[%s]", p.binlog.Name, p.binlog.Pos, p.gtidSet)
}

// SourceConfig stores the configuration for the MySQL Event Source.
type SourceConfig struct {
	Connection replication.BinlogSyncerConfig
	Databases  []struct {
		Tables []string
	}
}

// NewConfig creates a new SourceConfig
func NewConfig() SourceConfig {
	return SourceConfig{}
}

// A Source is an EventSource that streams events from a MySQL binlog.
type Source struct {
	cfg    SourceConfig
	conn   *client.Conn
	syncer *replication.BinlogSyncer
}

// ConnectSource creates a new MySQLSource connected to the server specified in the provided configuration.
func ConnectSource(cfg SourceConfig) (*Source, error) {
	var addr string
	if cfg.Connection.Port != 0 {
		addr = fmt.Sprintf("%s:%d", cfg.Connection.Host, cfg.Connection.Port)
	} else {
		addr = cfg.Connection.Host
	}
	conn, err := client.Connect(addr, cfg.Connection.User, cfg.Connection.Password, "")
	if err != nil {
		return nil, fmt.Errorf("error connecting to MySQL: %v", err)
	}

	// TODO: Deserialize stored schema

	syncer := replication.NewBinlogSyncer(cfg.Connection)
	return &Source{cfg, conn, syncer}, nil
}

// CaptureSchema fetches the database schema for the requested table.
func (s *Source) captureSchema(database, name string) (*schema.Table, error) {
	table, err := schema.NewTable(s.conn, database, name)
	if err != nil {
		return nil, err
	}
	return table, nil
}

// StartStream starts a new binlog stream from the specified position.
func (s *Source) StartStream(position stream.Position) (stream.EventStream, error) {
	mysqlPos, ok := position.(*mySQLPosition)
	if !ok {
		return nil, fmt.Errorf("position is not a valid MySQL position")
	}

	if mysqlPos.hasGtid {
		streamer, err := s.syncer.StartSyncGTID(mysqlPos.gtidSet)
		if err != nil {
			return nil, fmt.Errorf("error starting binlog connection: %v", err)
		}
		return &mySQLStream{streamer}, nil
	} else {
		streamer, err := s.syncer.StartSync(mysqlPos.binlog)
		if err != nil {
			return nil, fmt.Errorf("error starting binlog connection: %v", err)
		}
		return &mySQLStream{streamer}, nil
	}
}

// GetCurrentPosition retrieves the position representing the next (currently unwritten) binlog entry will be in.
func (s *Source) GetCurrentPosition() (stream.Position, error) {
	log.Debug("Running 'SHOW MASTER STATUS'")
	result, err := s.conn.Execute("SHOW MASTER STATUS")
	if err != nil {
		return nil, fmt.Errorf("error requesting master status: %v", err)
	}
	defer result.Close()

	binlogFile, err := result.GetStringByName(0, "File")
	if err != nil {
		return nil, fmt.Errorf("error reading master status: %v", err)
	}
	binlogPos, err := result.GetUintByName(0, "Position")
	if err != nil {
		return nil, fmt.Errorf("error reading master status: %v", err)
	}
	gtidSetString, err := result.GetStringByName(0, "Executed_Gtid_Set")
	if err != nil {
		return nil, fmt.Errorf("error reading master status: %v", err)
	}

	hasGtid := false
	var gtidSet mysql.GTIDSet
	if len(gtidSetString) >= 0 {
		gtidSet, err = mysql.ParseGTIDSet(s.cfg.Flavor, gtidSetString)
		if err != nil {
			return nil, fmt.Errorf("error parsing GTID set '%s': %v", gtidSetString, err)
		}
		hasGtid = true
	}

	pos := &mySQLPosition{hasGtid, mysql.Position{Name: binlogFile, Pos: uint32(binlogPos)}, gtidSet}

	log.Debugf("Identified current position: %s", pos)
	return pos, nil
}

type mySQLStream struct {
	streamer *replication.BinlogStreamer
}

// NextEvent returns the next event in the stream.
func (s *mySQLStream) NextEvent(ctx context.Context) (stream.Event, error) {
	evt, err := s.streamer.GetEvent(ctx)
	if err != nil {
		return nil, fmt.Errorf("error fetching next event: %v", err)
	}
	typ := reflect.TypeOf(evt.Event)
	log.Debugf("Retrieved %s event (type: %s)", evt.Header.EventType, typ)
	return evt.Event, nil
}
