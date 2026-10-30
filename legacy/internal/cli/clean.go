package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

func newCleanCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "clean",
		Short: "Remove build artifacts",
		Long:  "Remove build artifacts and temporary files",
		Run: func(cmd *cobra.Command, args []string) {
			if verbose {
				fmt.Println("Cleaning build artifacts...")
			}

			// Common build artifacts to clean
			artifacts := []string{
				"target",
				"*.exe",
				"*.out",
				"a.out",
			}

			removed := 0
			for _, pattern := range artifacts {
				matches, err := filepath.Glob(pattern)
				if err != nil {
					continue
				}

				for _, match := range matches {
					if err := os.RemoveAll(match); err == nil {
						if verbose {
							fmt.Printf("Removed: %s\n", match)
						}
						removed++
					}
				}
			}

			if removed > 0 {
				fmt.Printf("Cleaned %d build artifacts\n", removed)
			} else {
				fmt.Println("No build artifacts found")
			}
		},
	}
}
