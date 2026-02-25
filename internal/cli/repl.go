package cli

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/ecocee/kode-go/internal/version"
	"github.com/spf13/cobra"
)

func newReplCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "repl",
		Short: "Start interactive shell",
		Long:  "Start an interactive Kode REPL (Read-Eval-Print Loop)",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Printf("Kode REPL %s\n", version.Version)
			fmt.Println("Type 'exit' or 'quit' to exit")
			fmt.Println()

			scanner := bufio.NewScanner(os.Stdin)
			fmt.Print("> ")

			for scanner.Scan() {
				input := strings.TrimSpace(scanner.Text())

				if input == "exit" || input == "quit" {
					fmt.Println("Goodbye!")
					break
				}

				if input == "" {
					fmt.Print("> ")
					continue
				}

				// TODO: Implement actual REPL evaluation
				fmt.Printf("Echo: %s\n", input)
				fmt.Print("> ")
			}

			if err := scanner.Err(); err != nil {
				fmt.Fprintf(os.Stderr, "Error reading input: %v\n", err)
			}
		},
	}
}
