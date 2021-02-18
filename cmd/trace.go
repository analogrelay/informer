package cmd

import (
	"context"
	"fmt"
	"math/rand"
	"os"

	"github.com/anurse/informer/mysql"
	go_mysql "github.com/siddontang/go-mysql/mysql"
	"github.com/siddontang/go-mysql/replication"
	"github.com/spf13/cobra"
)

type traceCommand struct {
	host     string
	user     string
	port     uint16
	password string
	flavor   string
	serverID uint32
}

// NewTraceCommand creates an instance of the trace command.
func NewTraceCommand() *cobra.Command {
	var (
		trace traceCommand
		cmd   = &cobra.Command{
			Use:   "trace [tables...]",
			Short: "Trace events occurring in the specified tables.",
			Long: `Trace events occurring in the specified tables.

Tables are specified as '[database].[table]', where [database] is the name of the database and [table] is the name of the table.`,
			Args: cobra.MaximumNArgs(1),
			RunE: trace.Run,
		}
	)

	trace.registerFlags(cmd)
	return cmd
}

func init() {
	rootCommand.AddCommand(NewTraceCommand())
}

func (t *traceCommand) registerFlags(cmd *cobra.Command) {
	cmd.Flags().StringVarP(&t.host, "host", "H", "localhost", "The MySQL host to connect to.")
	cmd.Flags().StringVarP(&t.user, "user", "u", "root", "The user name to use to connect to MySQL.")
	cmd.Flags().Uint16VarP(&t.port, "port", "P", 3306, "The port to use to connect to MySQL.")
	cmd.Flags().StringVarP(&t.password, "password", "p", "", "The password to use to connect to MySQL.")
	cmd.Flags().StringVarP(&t.flavor, "flavor", "f", go_mysql.MySQLFlavor, "The 'flavor' of the server, either 'mysql' or 'mariadb' (defaults to 'mysql').")
	cmd.Flags().Uint32Var(&t.serverID, "server-id", 0, "The server_id to use when registering as a MySQL replica.")
}

func (t *traceCommand) Run(cmd *cobra.Command, args []string) error {
	if t.serverID == 0 {
		t.serverID = rand.Uint32()
	}

	// First we need to get the current master position
	cfg := replication.BinlogSyncerConfig{
		Host:     t.host,
		User:     t.user,
		Port:     t.port,
		Password: t.password,
		Flavor:   t.flavor,
		ServerID: t.serverID,
	}

	src, err := mysql.ConnectSource(cfg)
	if err != nil {
		return fmt.Errorf("error connecting MySQL source: %v", err)
	}

	pos, err := src.GetCurrentPosition()
	if err != nil {
		return fmt.Errorf("error getting current position: %v", err)
	}

	strm, err := src.StartStream(pos)
	if err != nil {
		return fmt.Errorf("error starting stream: %v", err)
	}

	evt, err := strm.NextEvent(context.Background())
	for err == nil {
		evt.Dump(os.Stdout)
		evt, err = strm.NextEvent(context.Background())
	}

	return nil
}
