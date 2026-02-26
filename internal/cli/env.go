package cli

import (
	"fmt"
	"os"
	"runtime"

	"github.com/ecocee/kode-go/internal/version"
	"github.com/spf13/cobra"
)

func newEnvCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "env",
		Short: "Show compiler environment",
		Long:  "Display information about the compiler environment",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("Kode Environment Information")
			fmt.Println("============================")
			fmt.Printf("Kode Version:    %s\n", version.Version)
			fmt.Printf("Go Version:      %s\n", runtime.Version())
			fmt.Printf("OS:              %s\n", runtime.GOOS)
			fmt.Printf("Architecture:    %s\n", runtime.GOARCH)
			fmt.Printf("GOROOT:          %s\n", runtime.GOROOT())
			if gopath := os.Getenv("GOPATH"); gopath != "" {
				fmt.Printf("GOPATH:          %s\n", gopath)
			} else {
				fmt.Println("GOPATH:          (not set)")
			}
			fmt.Printf("Working Dir:     %s\n", getWd())
		},
	}
}

func getWd() string {
	if wd, err := os.Getwd(); err == nil {
		return wd
	}
	return "(unknown)"
}
