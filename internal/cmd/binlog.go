package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

var binlogCommand = &cobra.Command{
	Use:   "trace",
	Short: "Trace events occurring in a database.",
	Long:  `Traces events occurring in a database, writing them to the console or a file.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Tracing...")
	},
}

func init() {
	rootCommand.AddCommand(binlogCommand)
}
