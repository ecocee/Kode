package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var (
	verbose bool
	quiet   bool
	config  string
)

// NewRootCmd creates the root command
func NewRootCmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:   "kode",
		Short: "Kode Programming Language CLI",
		Long: `Kode Programming Language CLI

A modern systems programming language for backend services and AI tooling.`,
		Run: func(cmd *cobra.Command, args []string) {
			cmd.Help()
		},
	}

	// Global flags
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	rootCmd.PersistentFlags().BoolVarP(&quiet, "quiet", "q", false, "quiet output")
	rootCmd.PersistentFlags().StringVar(&config, "config", "", "config file")

	// Add subcommands
	rootCmd.AddCommand(
		newRunCmd(),
		newBuildCmd(),
		newCheckCmd(),
		newFmtCmd(),
		newReplCmd(),
		newNewCmd(),
		newCleanCmd(),
		newEnvCmd(),
		newDoctorCmd(),
		newVersionCmd(),
	)

	return rootCmd
}

// Execute runs the root command
func Execute() {
	if err := NewRootCmd().Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
