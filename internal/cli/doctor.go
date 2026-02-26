package cli

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/spf13/cobra"
)

func newDoctorCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "doctor",
		Short: "Diagnose installation issues",
		Long:  "Check system for potential issues with Kode installation",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("Kode Doctor")
			fmt.Println("===========")

			checks := []struct {
				name string
				fn   func() (bool, string)
			}{
				{"Go installation", checkGoInstalled},
				{"Go version", checkGoVersion},
				{"PATH configuration", checkPath},
				{"Write permissions", checkWritePermissions},
			}

			passed := 0
			total := len(checks)

			for _, check := range checks {
				ok, msg := check.fn()
				if ok {
					fmt.Printf("✓ %s: %s\n", check.name, msg)
					passed++
				} else {
					fmt.Printf("✗ %s: %s\n", check.name, msg)
				}
			}

			fmt.Printf("\nSummary: %d/%d checks passed\n", passed, total)

			if passed == total {
				fmt.Println("🎉 Your Kode installation looks good!")
			} else {
				fmt.Println("Some issues were found. Please address them for optimal Kode experience.")
			}
		},
	}
}

func checkGoInstalled() (bool, string) {
	cmd := exec.Command("go", "version")
	if err := cmd.Run(); err != nil {
		return false, "Go is not installed or not in PATH"
	}
	return true, "Go is installed"
}

func checkGoVersion() (bool, string) {
	cmd := exec.Command("go", "version")
	output, err := cmd.Output()
	if err != nil {
		return false, "Could not determine Go version"
	}

	version := strings.TrimSpace(string(output))
	if strings.Contains(version, "go1.21") || strings.Contains(version, "go1.22") || strings.Contains(version, "go1.23") || strings.Contains(version, "go1.24") || strings.Contains(version, "go1.25") {
		return true, fmt.Sprintf("Compatible Go version found: %s", version)
	}
	return false, fmt.Sprintf("Go version may be too old: %s (recommended: 1.21+)", version)
}

func checkPath() (bool, string) {
	path := os.Getenv("PATH")
	goBin := filepath.Join(runtime.GOROOT(), "bin")

	if strings.Contains(path, goBin) {
		return true, "Go binary directory is in PATH"
	}

	// Check if 'go' command works anyway
	cmd := exec.Command("go", "version")
	if err := cmd.Run(); err == nil {
		return true, "Go command works (may be in different location)"
	}

	return false, "Go binary directory not found in PATH"
}

func checkWritePermissions() (bool, string) {
	wd, err := os.Getwd()
	if err != nil {
		return false, "Could not determine working directory"
	}

	testFile := filepath.Join(wd, ".kode-doctor-test")
	if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
		return false, "No write permissions in current directory"
	}

	os.Remove(testFile)
	return true, "Write permissions OK"
}
