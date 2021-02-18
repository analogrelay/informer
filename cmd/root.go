package cmd

import (
	"fmt"
	"os"

	"github.com/siddontang/go-log/log"
	"github.com/spf13/cobra"
)

var rootCommand = &cobra.Command{
	Use:   "informer",
	Short: "Spying on your DB",
	Long:  `A Change Data Capture tool for monitoring a Database and publishing events when changes occur.`,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		h, err := log.NewStreamHandler(os.Stdout)
		if err != nil {
			panic(err)
		}
		l := log.NewDefault(h)
		log.SetDefaultLogger(l)
		if verbose {
			l.SetLevel(log.LevelDebug)
		} else {
			l.SetLevel(log.LevelInfo)
		}
	},
	Run: func(cmd *cobra.Command, args []string) {
		cmd.Help()
	},
}

var verbose bool = false

func init() {
	rootCommand.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
}

// Execute launches the informer command line tool.
func Execute() {
	if err := rootCommand.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
