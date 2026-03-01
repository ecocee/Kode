package cli

import (
	"fmt"
	"os"
	"strings"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/bytecode"
	"github.com/spf13/cobra"
)

// displayError formats and displays an error with file/line information
func displayError(err error) {
	if kodeErr, ok := err.(interface{ Error() string }); ok {
		// Check if it's our KodeError type
		if strings.Contains(kodeErr.Error(), ":") {
			parts := strings.SplitN(kodeErr.Error(), ":", 3)
			if len(parts) >= 2 {
				file := parts[0]
				if len(parts) == 3 {
					line := parts[1]
					message := parts[2]
					fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %s:%s: %s\n", file, line, strings.TrimSpace(message))
				} else {
					fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %s: %s\n", file, strings.TrimSpace(parts[1]))
				}
				return
			}
		}
	}
	// Fallback for regular errors
	fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
}

var (
	verbose bool
	quiet   bool
	config  string
)

// NewRootCmd creates the root command
func NewRootCmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:   "kode",
		Short: "Kode Programming Language CLI",
		Long: `Kode Programming Language CLI

A modern systems programming language for backend services and AI tooling.`,
		Run: func(cmd *cobra.Command, args []string) {
			// If a single file argument is provided, try to run it directly
			if len(args) == 1 {
				file := args[0]
				if strings.HasSuffix(file, ".kode") || strings.HasSuffix(file, ".kbc") {
					runFile(file)
					return
				}
			}
			// Otherwise show help
			cmd.Help()
		},
	}

	// Global flags
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	rootCmd.PersistentFlags().BoolVarP(&quiet, "quiet", "q", false, "quiet output")
	rootCmd.PersistentFlags().StringVar(&config, "config", "", "config file")

	// Add subcommands
	rootCmd.AddCommand(
		newRunCmd(),
		newBuildCmd(),
		newCheckCmd(),
		newFmtCmd(),
		newNewCmd(),
		newCleanCmd(),
		newEnvCmd(),
		newDoctorCmd(),
		newVersionCmd(),
	)

	return rootCmd
}

// runFile executes a .kode or .kbc file directly
func runFile(file string) {
	if verbose {
		fmt.Printf("Running file: %s\n", file)
	}

	// Check file extension to determine if it's source or bytecode
	if strings.HasSuffix(file, ".kbc") {
		// Handle bytecode file
		if verbose {
			fmt.Printf("Loading bytecode file: %s\n", file)
		}

		// Read bytecode file
		data, err := os.ReadFile(file)
		if err != nil {
			fmt.Fprintf(os.Stderr, "\033[1;31m✗ File Error\033[0m\n")
			fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
			os.Exit(1)
		}

		// Deserialize bytecode
		prog, err := bytecode.Deserialize(data)
		if err != nil {
			fmt.Fprintf(os.Stderr, "\033[1;31m✗ Bytecode Error\033[0m in %s\n", file)
			fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
			os.Exit(1)
		}

		if verbose {
			fmt.Printf("Loaded bytecode: %d instructions\n", len(prog.Instructions))
			fmt.Printf("Constants: %d\n", len(prog.Constants))
			fmt.Printf("Globals: %d\n", len(prog.Globals))
		}

		// Create and run VM
		vm := bytecode.NewVM(prog)
		if err := vm.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "\033[1;31m✗ Execution Error\033[0m\n")
			fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
			os.Exit(1)
		}

		if verbose {
			fmt.Println("Bytecode execution completed")
		}
		return
	}

	// Handle source file (.kode)
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

	// Compile IR to bytecode
	bc := bytecode.NewCompiler()
	prog, err := bc.Compile(ir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "\033[1;31m✗ Bytecode Compilation Error\033[0m in %s\n", file)
		fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
		os.Exit(1)
	}

	// Execute bytecode on VM
	vm := bytecode.NewVM(prog)
	if err := vm.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "\033[1;31m✗ Execution Error\033[0m\n")
		fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
		os.Exit(1)
	}
}

// Execute runs the root command
func Execute() {
	// Check if the first argument looks like a Kode file
	if len(os.Args) == 2 && (strings.HasSuffix(os.Args[1], ".kode") || strings.HasSuffix(os.Args[1], ".kbc")) {
		runFile(os.Args[1])
		return
	}

	if err := NewRootCmd().Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

// requireArgs creates a custom Args function with user-friendly error messages
func requireArgs(count int, usage string) cobra.PositionalArgs {
	return func(cmd *cobra.Command, args []string) error {
		if len(args) != count {
			if count == 1 {
				return fmt.Errorf("please provide %s\n\nUsage:\n  %s", usage, cmd.UseLine())
			}
			return fmt.Errorf("expected %d arguments, got %d\n\nUsage:\n  %s", count, len(args), cmd.UseLine())
		}
		return nil
	}
}
