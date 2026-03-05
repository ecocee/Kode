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
	"github.com/spf13/cobra"
)

// resolveImports replaces ImportStmt nodes in stmts with the actual function
// definitions from the imported file, so they are available at compile time.
// baseDir is the directory containing the importing file.
func resolveImports(baseDir string, stmts []ast.Statement) ([]ast.Statement, error) {
	var result []ast.Statement

	for _, stmt := range stmts {
		imp, ok := stmt.(ast.ImportStmt)
		if !ok {
			result = append(result, stmt)
			continue
		}

		// Resolve the file path: add .kode if missing
		modPath := imp.Path
		if !strings.HasSuffix(modPath, ".kode") {
			modPath += ".kode"
		}

		// Normalise: strip leading "./" so "./stdlib/server" == "stdlib/server"
		normPath := strings.TrimPrefix(modPath, "./")

		// Also derive a bare name (strip any "stdlib/" prefix) so that
		// "stdlib/server", "./stdlib/server", and "server" all resolve to
		// stdlib/server.kode when looked up in the stdlib dir.
		bareName := normPath
		if strings.HasPrefix(bareName, "stdlib/") || strings.HasPrefix(bareName, "stdlib\\") {
			bareName = bareName[len("stdlib/"):]
		}

		// Candidate resolution order:
		//   1. stdlib/<bareName>          (project-root stdlib dir, highest priority for stdlib imports)
		//   2. relative to the importing file's directory
		//   3. cwd-relative path as-is    (handles absolute or explicit cwd paths)
		var resolvedPath string
		candidates := []string{
			filepath.Join("stdlib", bareName),
			filepath.Join(baseDir, normPath),
			normPath,
		}
		for _, cand := range candidates {
			if _, err := os.Stat(cand); err == nil {
				resolvedPath = cand
				break
			}
		}
		if resolvedPath == "" {
			return nil, fmt.Errorf("cannot import %q: module not found (searched stdlib/ and relative to %s)", imp.Path, baseDir)
		}
		modPath = resolvedPath

		src, err := os.ReadFile(modPath)
		if err != nil {
			return nil, fmt.Errorf("cannot import %q: %v", imp.Path, err)
		}

		p, err := parser.NewParser(modPath, string(src))
		if err != nil {
			return nil, fmt.Errorf("import %q lexer error: %v", imp.Path, err)
		}
		importedStmts, err := p.Parse()
		if err != nil {
			return nil, fmt.Errorf("import %q parse error: %v", imp.Path, err)
		}

		// Build a set of allowed names (empty = allow all)
		allowed := make(map[string]bool)
		for _, name := range imp.Items {
			allowed[name] = true
		}

		// Collect exported function definitions from the imported file
		for _, is := range importedStmts {
			switch s := is.(type) {
			case ast.FunctionDefStmt:
				if len(allowed) == 0 || allowed[s.Name] {
					result = append(result, s)
				}
			case ast.ExportStmt:
				if fn, ok := s.Statement.(ast.FunctionDefStmt); ok {
					if len(allowed) == 0 || allowed[fn.Name] {
						result = append(result, fn)
					}
				}
			}
		}
	}

	return result, nil
}

func newRunCmd() *cobra.Command {
	var release bool

	cmd := &cobra.Command{
		Use:   "run <file>",
		Short: "Run a Kode source file",
		Long:  "Compile and execute a Kode source file (.kode only)",
		Args:  requireArgs(1, "a Kode source file to run (e.g., 'kode main.kode')"),
		Run: func(cmd *cobra.Command, args []string) {
			file := args[0]

			if verbose {
				fmt.Printf("Running file: %s\n", file)
			}

			if release {
				fmt.Println("Running in release mode")
			}

			// Check if it's a bytecode file - direct execution should be used instead
			if strings.HasSuffix(file, ".kbc") {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Error\033[0m: Use 'kode %s' to run bytecode files directly\n", file)
				os.Exit(1)
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

			// Resolve imports — splice imported function definitions into the program
			baseDir := filepath.Dir(file)
			statements, err = resolveImports(baseDir, statements)
			if err != nil {
				fmt.Fprintf(os.Stderr, "\033[1;31m✗ Import Error\033[0m in %s\n", file)
				fmt.Fprintf(os.Stderr, "  \033[1;33m→\033[0m %v\n", err)
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
		},
	}

	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")

	return cmd
}
