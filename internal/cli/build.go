package cli

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/ecocee/kode-go/internal/codegen"
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
	var useLLVM bool
	var useGo bool

	cmd := &cobra.Command{
		Use:   "build <file>",
		Short: "Compile a Kode file",
		Long:  "Compile a Kode source file to an executable",
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

			// Choose backend (bytecode is default)
			if useLLVM {
				buildWithLLVM(ir, output, file, verbose)
			} else if useGo {
				buildWithGo(ir, output, file, verbose)
			} else {
				// Bytecode is the default
				buildWithBytecode(ir, output, verbose)
			}
		},
	}

	cmd.Flags().StringVarP(&output, "output", "o", "", "output file name")
	cmd.Flags().BoolVar(&release, "release", false, "build in release mode")
	cmd.Flags().StringVar(&target, "target", "", "target platform")
	cmd.Flags().BoolVar(&useLLVM, "llvm", false, "use LLVM backend for compilation")
	cmd.Flags().BoolVar(&useGo, "go", false, "use Go backend for compilation (legacy, default is bytecode)")

	return cmd
}

// buildWithGo compiles using the Go backend
func buildWithGo(irProg *ir.IR, output string, sourceFile string, verbose bool) {
	// Generate Go code from IR
	codeGen := codegen.NewGoCodeGenerator(irProg)
	codeGen.SetSourceFile(sourceFile)
	goCode, err := codeGen.Generate()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error generating code: %v\n", err)
		os.Exit(1)
	}

	// Write generated Go code to temporary file
	tempDir := os.TempDir()
	tempFile := filepath.Join(tempDir, "kode_generated.go")
	err = ioutil.WriteFile(tempFile, []byte(goCode), 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error writing temporary file: %v\n", err)
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Generated Go code: %s\n", tempFile)
		fmt.Printf("Generated Go code length: %d bytes\n", len(goCode))
		if len(goCode) > 500 {
			fmt.Printf("First 500 chars:\n%s\n", goCode[:500])
		} else {
			fmt.Printf("Full generated code:\n%s\n", goCode)
		}
	}

	// Compile Go code to executable using go build
	outputFile := output
	if filepath.Ext(outputFile) == "" {
		outputFile += ".exe"
	}

	// remove any stale output so we can't accidentally leave a script behind
	if err := os.Remove(outputFile); err == nil {
		if verbose {
			fmt.Printf("Removed existing output file: %s\n", outputFile)
		}
	}

	buildCmd := exec.Command("go", "build", "-o", outputFile, tempFile)
	buildCmd.Stdout = os.Stdout
	buildCmd.Stderr = os.Stderr

	if verbose {
		fmt.Printf("Running: go build -o %s %s\n", outputFile, tempFile)
	}

	err = buildCmd.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error building executable: %v\n", err)
		fmt.Fprintf(os.Stderr, "Generated Go file: %s\n", tempFile)
		fmt.Fprintf(os.Stderr, "Generated Go file contents:\n")
		if goCodeBytes, readErr := ioutil.ReadFile(tempFile); readErr == nil {
			fmt.Fprintf(os.Stderr, string(goCodeBytes))
		}
		// if the output file was created and is tiny, remove it to avoid confusion
		if fi, statErr := os.Stat(outputFile); statErr == nil && fi.Size() < 1024 {
			os.Remove(outputFile)
		}
		os.Exit(1)
	}

	// sanity check: make sure the produced file isn't just a script/shebang
	if data, readErr := os.ReadFile(outputFile); readErr == nil {
		if len(data) > 0 && strings.HasPrefix(string(data), "#!") {
			fmt.Fprintf(os.Stderr, "build succeeded but output file '%s' appears to contain a shebang/script instead of a binary\n", outputFile)
			size := len(data)
			fmt.Fprintf(os.Stderr, "(first %d bytes):\n%s\n", size, string(data))
			os.Remove(outputFile)
			os.Exit(1)
		}
	}

	// Copy the source file to the same directory as the executable
	// so it can be executed at runtime
	outputDir := filepath.Dir(outputFile)
	if outputDir == "" {
		outputDir = "."
	}
	sourceFileName := filepath.Base(sourceFile)
	destSourceFile := filepath.Join(outputDir, sourceFileName)

	sourceData, err := os.ReadFile(sourceFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading source file: %v\n", err)
		os.Exit(1)
	}

	err = os.WriteFile(destSourceFile, sourceData, 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error copying source file: %v\n", err)
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Successfully built executable: %s\n", outputFile)
		fmt.Printf("Copied source file to: %s\n", destSourceFile)
	} else {
		fmt.Printf("Successfully built: %s\n", outputFile)
	}

	os.Remove(tempFile)
}

// buildWithLLVM compiles using the LLVM backend
func buildWithLLVM(irProg *ir.IR, output string, _ string, verbose bool) {
	// Generate LLVM IR
	llvmGen := codegen.NewLLVMCodeGenerator(irProg)
	llvmCode, err := llvmGen.Generate()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error generating LLVM code: %v\n", err)
		os.Exit(1)
	}

	// Write generated LLVM IR to temporary file
	tempDir := os.TempDir()
	llvmFile := filepath.Join(tempDir, "kode_generated.ll")
	err = ioutil.WriteFile(llvmFile, []byte(llvmCode), 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error writing LLVM file: %v\n", err)
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Generated LLVM IR: %s\n", llvmFile)
		fmt.Printf("Generated LLVM code length: %d bytes\n", len(llvmCode))
		if len(llvmCode) > 500 {
			fmt.Printf("First 500 chars:\n%s\n", llvmCode[:500])
		} else {
			fmt.Printf("Full generated LLVM code:\n%s\n", llvmCode)
		}
	}

	// remove any stale output before linking
	if err := os.Remove(output); err == nil && verbose {
		fmt.Printf("Removed existing output file: %s\n", output)
	}

	// Compile LLVM IR to object file using llc
	objFile := filepath.Join(tempDir, "kode_generated.o")
	llcCmd := exec.Command("llc", "-relocation-model=pic", "-filetype=obj", "-o", objFile, llvmFile)
	llcCmd.Stdout = os.Stdout
	llcCmd.Stderr = os.Stderr

	if verbose {
		fmt.Printf("Running: llc -relocation-model=pic -filetype=obj -o %s %s\n", objFile, llvmFile)
	}

	err = llcCmd.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error compiling with llc: %v\n", err)
		fmt.Fprintf(os.Stderr, "Make sure LLVM is installed and llc is in your PATH\n")
		fmt.Fprintf(os.Stderr, "Generated LLVM file: %s\n", llvmFile)
		os.Exit(1)
	}

	// Link object file to executable using clang
	outputFile := output
	if filepath.Ext(outputFile) == "" {
		outputFile += ".exe"
	}

	linkCmd := exec.Command("clang", "-o", outputFile, objFile)
	linkCmd.Stdout = os.Stdout
	linkCmd.Stderr = os.Stderr

	if verbose {
		fmt.Printf("Running: clang -o %s %s\n", outputFile, objFile)
	}

	err = linkCmd.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error linking with clang: %v\n", err)
		fmt.Fprintf(os.Stderr, "Make sure Clang is installed and in your PATH\n")
		os.Exit(1)
	}

	if verbose {
		fmt.Printf("Successfully built executable: %s\n", outputFile)
	} else {
		fmt.Printf("Successfully built: %s\n", outputFile)
	}

	// sanity check on generated file
	if data, readErr := os.ReadFile(outputFile); readErr == nil {
		if len(data) > 0 && strings.HasPrefix(string(data), "#!") {
			fmt.Fprintf(os.Stderr, "build succeeded but output file '%s' appears to contain a shebang/script instead of a binary\n", outputFile)
			os.Remove(outputFile)
			os.Exit(1)
		}
	}

	// Clean up temporary files
	os.Remove(llvmFile)
	os.Remove(objFile)
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
