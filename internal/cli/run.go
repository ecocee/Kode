package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

func newRunCmd() *cobra.Command {
	var release bool

	cmd := &cobra.Command{
		Use:   "run <file>",
		Short: "Run a Kode file",
		Long:  "Compile and execute a Kode source file",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Running file: %s\n", file)
			}

			if release {
				fmt.Println("Running in release mode")
			}

			// TODO: Implement actual compilation and execution
			fmt.Printf("Would run: %s\n", file)
		},
	}

	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")

	return cmd
}
