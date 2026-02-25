package cli

import (
	"fmt"
	"os"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/runtime"
	"github.com/spf13/cobra"
)

func newRunCmd() *cobra.Command {
	var release bool

	cmd := &cobra.Command{
		Use:   "run <file>",
		Short: "Run a Kode file",
		Long:  "Compile and execute a Kode source file",
		Args:  requireArgs(1, "a Kode file to run (e.g., 'kode run main.kode')"),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Running file: %s\n", file)
			}

			if release {
				fmt.Println("Running in release mode")
			}

			// Read the file
			sourceCode, err := os.ReadFile(file)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error reading file: %v\n", err)
				os.Exit(1)
			}

			// Parse the code
			p, err := parser.NewParser(file, string(sourceCode))
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error creating parser: %v\n", err)
				os.Exit(1)
			}

			statements, err := p.Parse()
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error parsing file: %v\n", err)
				os.Exit(1)
			}

			// Compile to IR
			c := compiler.NewCompiler()
			ir, err := c.Compile(ast.Program{Statements: statements})
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error compiling file: %v\n", err)
				os.Exit(1)
			}

			// Execute
			rt := runtime.NewRuntime()
			if err := rt.Execute(ir); err != nil {
				fmt.Fprintf(os.Stderr, "Error executing file: %v\n", err)
				os.Exit(1)
			}
		},
	}

	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")

	return cmd
}
