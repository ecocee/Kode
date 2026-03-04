package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/bytecode"
	"github.com/ecocee/kode-go/pkg/ir"
	"github.com/spf13/cobra"
)

func newBuildCmd() *cobra.Command {
	var output string
	var release bool
	var target string

	cmd := &cobra.Command{
		Use:   "build <file>",
		Short: "Compile a Kode file",
		Long:  "Compile a Kode source file to bytecode",
		Args:  requireArgs(1, "a Kode file to build (e.g., 'kode build main.kode')"),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("\033[1;36m⏳ Building file:\033[0m \033[1;33m%s\033[0m\n", file)
			}

			// Set default output name
			if output == "" {
				baseName := filepath.Base(file)
				if filepath.Ext(baseName) == ".kode" {
					output = baseName[:len(baseName)-5] // Remove .kode extension
				} else {
					output = baseName
				}

			}

			// Read the file
			sourceCode, err := os.ReadFile(file)
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Error reading file:\033[0m %v\n", err)
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
				displayError(err)
				os.Exit(1)
			}

			// Compile to IR
			c := compiler.NewCompiler()
			ir, err := c.Compile(ast.Program{Statements: statements})
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Compiler Error\033[0m in %s\n", file)
				displayError(err)
				os.Exit(1)
			}

			// Compile to bytecode (only backend now)
			buildWithBytecode(ir, output, verbose)
		},
	}

	cmd.Flags().StringVarP(&output, "output", "o", "", "output file name")
	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")
	cmd.Flags().StringVar(&target, "target", "", "target platform")

	return cmd
}

// buildWithBytecode compiles using bytecode
func buildWithBytecode(irProg *ir.IR, output string, verbose bool) {
	// Compile IR to bytecode
	bc := bytecode.NewCompiler()
	prog, err := bc.Compile(irProg)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error compiling to bytecode: %v\n", err)
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Compiled to bytecode: %d instructions\n", len(prog.Instructions))
		fmt.Printf("Constants: %v\n", prog.Constants)
		fmt.Printf("Globals: %v\n", prog.Globals)
	}

	// Serialize bytecode
	bytecodeData, err := prog.Serialize()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error serializing bytecode: %v\n", err)
		os.Exit(1)
	}

	// Write bytecode file with .kbc extension
	bytecodeFile := strings.TrimSuffix(output, ".exe")
	if !strings.HasSuffix(bytecodeFile, ".kbc") {
		bytecodeFile += ".kbc"
	}
	err = os.WriteFile(bytecodeFile, bytecodeData, 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error writing bytecode file: %v\n", err)
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Bytecode file size: %d bytes\n", len(bytecodeData))
		fmt.Printf("Bytecode written to: %s\n", bytecodeFile)
	}

	// Create a wrapper executable that reads and executes the bytecode
	// For now, just indicate how to run it
	fmt.Printf("Successfully built: %s\n", bytecodeFile)
	fmt.Printf("Run with: kode %s\n", bytecodeFile)
}
