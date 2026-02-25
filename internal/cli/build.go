package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

func newBuildCmd() *cobra.Command {
	var output string
	var release bool
	var target string

	cmd := &cobra.Command{
		Use:   "build <file>",
		Short: "Compile a Kode file",
		Long:  "Compile a Kode source file to an executable",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Building file: %s\n", file)
			}

			if output == "" {
				output = "a.out" // default
			}

			if release {
				fmt.Println("Building in release mode")
			}

			if target != "" {
				fmt.Printf("Target platform: %s\n", target)
			}

			// TODO: Implement actual compilation
			fmt.Printf("Would build: %s -> %s\n", file, output)
		},
	}

	cmd.Flags().StringVarP(&output, "output", "o", "", "output file name")
	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")
	cmd.Flags().StringVar(&target, "target", "", "target platform")

	return cmd
}
