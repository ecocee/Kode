package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

func newCheckCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "check <file>",
		Short: "Type-check only",
		Long:  "Type-check a Kode source file without generating output",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Checking file: %s\n", file)
			}

			// TODO: Implement type checking
			fmt.Printf("Would check: %s\n", file)
		},
	}
}
