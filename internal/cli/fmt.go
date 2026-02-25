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
		Args:  requireArgs(1, "a Kode file to format (e.g., 'kode fmt main.kode')"),
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
