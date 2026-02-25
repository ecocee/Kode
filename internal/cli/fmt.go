package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

func newFmtCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "fmt <file>",
		Short: "Format source files",
		Long:  "Format Kode source files according to standard style",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Formatting file: %s\n", file)
			}

			// TODO: Implement formatting
			fmt.Printf("Would format: %s\n", file)
		},
	}
}
