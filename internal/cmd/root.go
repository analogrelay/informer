package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCommand = &cobra.Command{
	Use:   "informer",
	Short: "Spying on your DB",
	Long:  `A Change Data Capture tool for monitoring a Database and publishing events when changes occur.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Informing on your DB...")
	},
}

// Execute launches the informer command line tool.
func Execute() {
	if err := rootCommand.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
