package cli

import (
	"fmt"
	"os"

	"github.com/ecocee/kode-go/pkg/bytecode"
	"github.com/spf13/cobra"
)

func newExecCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "exec <bytecode>",
		Short: "Execute a bytecode file",
		Long:  "Execute a compiled Kode bytecode file",
		Args:  requireArgs(1, "a bytecode file to execute (e.g., 'kode program.bytecode')"),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Executing bytecode file: %s\n", file)
			}

			// Read bytecode file
			data, err := os.ReadFile(file)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error reading bytecode file: %v\n", err)
				os.Exit(1)
			}

			// Deserialize bytecode
			prog, err := bytecode.Deserialize(data)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error deserializing bytecode: %v\n", err)
				os.Exit(1)
			}

			if verbose {
				fmt.Printf("Loaded bytecode: %d instructions\n", len(prog.Instructions))
				fmt.Printf("Constants: %d\n", len(prog.Constants))
				fmt.Printf("Globals: %d\n", len(prog.Globals))
				if len(prog.Instructions) > 0 {
					fmt.Printf("First instruction: OpCode %d\n", prog.Instructions[0].Op)
				}
			}

			// Create and run VM
			vm := bytecode.NewVM(prog)
			if err := vm.Run(); err != nil {
				fmt.Fprintf(os.Stderr, "Runtime error: %v\n", err)
				os.Exit(1)
			}

			if verbose {
				fmt.Println("Bytecode execution completed")
			}
		},
	}

	return cmd
}
