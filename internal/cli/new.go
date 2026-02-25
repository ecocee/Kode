package cli

import (
	"fmt"
	"os"
	"path/filepath"

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

			// Create main.kode
			mainFile := filepath.Join(projectName, "main.kode")
			mainContent := fmt.Sprintf(`// %s - Main entry point
print("Hello from %s!")

func main() {
    print("Welcome to Kode!")
}
`, projectName, projectName)

			if err := os.WriteFile(mainFile, []byte(mainContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating main.kode: %v\n", err)
				os.Exit(1)
			}

			// Create kode.toml
			tomlFile := filepath.Join(projectName, "kode.toml")
			tomlContent := fmt.Sprintf(`[project]
name = "%s"
version = "0.1.0"
edition = "2026"

[dependencies]
`, projectName)

			if err := os.WriteFile(tomlFile, []byte(tomlContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating kode.toml: %v\n", err)
				os.Exit(1)
			}

			// Create .gitignore
			gitignoreFile := filepath.Join(projectName, ".gitignore")
			gitignoreContent := `# Build artifacts
target/
*.exe
*.out

# IDE files
.vscode/
.idea/

# OS files
.DS_Store
Thumbs.db
`

			if err := os.WriteFile(gitignoreFile, []byte(gitignoreContent), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error creating .gitignore: %v\n", err)
				os.Exit(1)
			}

			fmt.Printf("Created new Kode project: %s\n", projectName)
			fmt.Println("Project structure:")
			fmt.Printf("  %s/\n", projectName)
			fmt.Println("  ├── main.kode")
			fmt.Println("  ├── kode.toml")
			fmt.Println("  └── .gitignore")
		},
	}
}
