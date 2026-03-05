package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/spf13/cobra"
)

func newNewCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "new <project-name>",
		Short: "Create a new project",
		Long:  "Create a new Kode project with standard directory structure",
		Args:  requireArgs(1, "a project name (e.g., 'kode new myapp')"),
		Run: func(cmd *cobra.Command, args []string) {
			projectName := args[0]

			if verbose {
				fmt.Printf("Creating new project: %s\n", projectName)
			}

			// Create project directory
			if err := os.MkdirAll(projectName, 0755); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating directory: %v\n", err)
				os.Exit(1)
			}

			// Create src/ directory
			srcDir := filepath.Join(projectName, "src")
			if err := os.MkdirAll(srcDir, 0755); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating src directory: %v\n", err)
				os.Exit(1)
			}

			// Create main.kode
			mainFile := filepath.Join(projectName, "main.kode")
			mainContent := fmt.Sprintf(`// %s - main entry point
//
// Run:   kode run   (or: kode run main.kode)
// Build: kode build (produces target/main.kbc)
// Exec:  kode target/main.kbc

print("Hello from %s!")
`, projectName, projectName)

			if err := os.WriteFile(mainFile, []byte(mainContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating main.kode: %v\n", err)
				os.Exit(1)
			}

			// Create kode.toml
			year := time.Now().Year()
			tomlFile := filepath.Join(projectName, "kode.toml")
			tomlContent := fmt.Sprintf(`# ─────────────────────────────────────────────
# %s — Kode project manifest
# ─────────────────────────────────────────────

[project]
name        = "%s"
version     = "0.1.0"
edition     = "%d"
description = "A Kode project"
authors     = ["your name <you@example.com>"]
kode_version = "0.3.4"

# ── Build settings ────────────────────────────
# kode build         →  compiles entry → target/output.kbc
# kode build -o out  →  override output name
[build]
entry      = "main.kode"   # source file to compile
output     = "main"        # output name (without extension)
bytecode   = true          # emit .kbc bytecode file
target_dir = "target"      # output directory

# ── Run settings ──────────────────────────────
# kode run           →  runs entry directly (interpreted)
# kode run main.kode →  explicit file
[run]
entry = "main.kode"

# ── Script shortcuts ──────────────────────────
# (informational — used by editors/tooling)
[scripts]
run   = "kode run main.kode"
build = "kode build"
exec  = "kode target/main.kbc"
clean = "kode clean"

# ── Dependencies ──────────────────────────────
# (future: kode add <package>)
[dependencies]
`, projectName, projectName, year)

			if err := os.WriteFile(tomlFile, []byte(tomlContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating kode.toml: %v\n", err)
				os.Exit(1)
			}

			// Create .gitignore
			gitignoreFile := filepath.Join(projectName, ".gitignore")
			gitignoreContent := `# Kode build artifacts
target/
*.kbc

# Executables
*.exe
*.out

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
`

			if err := os.WriteFile(gitignoreFile, []byte(gitignoreContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating .gitignore: %v\n", err)
				os.Exit(1)
			}

			// Create README.md
			readmeFile := filepath.Join(projectName, "README.md")
			readmeContent := fmt.Sprintf(`# %s

A [Kode](https://github.com/ecocee/kode) project.

## Usage

`+"```"+`bash
# Run directly (interpreted)
kode run

# Build to bytecode
kode build

# Execute bytecode
kode target/main.kbc
`+"```"+`

## Project structure

`+"```"+`
%s/
├── main.kode       # entry point
├── src/            # additional source files
├── target/         # build output (.kbc files)
├── kode.toml       # project configuration
├── .gitignore
└── README.md
`+"```"+`
`, projectName, projectName)

			if err := os.WriteFile(readmeFile, []byte(readmeContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating README.md: %v\n", err)
				os.Exit(1)
			}

			fmt.Printf("\033[1;32m✓\033[0m Created new Kode project: \033[1;33m%s\033[0m\n", projectName)
			fmt.Println()
			fmt.Println("Project structure:")
			fmt.Printf("  \033[1;36m%s/\033[0m\n", projectName)
			fmt.Println("  ├── main.kode")
			fmt.Println("  ├── src/")
			fmt.Println("  ├── kode.toml")
			fmt.Println("  ├── .gitignore")
			fmt.Println("  └── README.md")
			fmt.Println()
			fmt.Println("Get started:")
			fmt.Printf("  \033[1;33mcd %s\033[0m\n", projectName)
			fmt.Printf("  \033[1;33mkode run\033[0m          # run directly\n")
			fmt.Printf("  \033[1;33mkode build\033[0m        # compile to target/main.kbc\n")
			fmt.Printf("  \033[1;33mkode target/main.kbc\033[0m  # run compiled bytecode\n")
		},
	}
}
