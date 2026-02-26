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
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ File Error\033[0m\n")
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
				os.Exit(1)
			}

			// Parse the code
			p, err := parser.NewParser(file, string(sourceCode))
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Lexer Error\033[0m in %s\n", file)
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
				os.Exit(1)
			}

			statements, err := p.Parse()
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Parser Error\033[0m in %s\n", file)
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
				os.Exit(1)
			}

			// Compile to IR
			c := compiler.NewCompiler()
			ir, err := c.Compile(ast.Program{Statements: statements})
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Compiler Error\033[0m in %s\n", file)
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
				os.Exit(1)
			}

			// Execute
			rt := runtime.NewRuntime()
			if err := rt.Execute(ir); err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Execution Error\033[0m\n")
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
				os.Exit(1)
			}
		},
	}

	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")

	return cmd
}
