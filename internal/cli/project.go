package cli

import (
	"bufio"
	"os"
	"strings"
)

// ProjectConfig holds configuration parsed from kode.toml.
type ProjectConfig struct {
	// [project]
	Name        string
	Version     string
	Edition     string
	Description string
	KodeVersion string

	// [build]
	BuildEntry  string // source file to compile (default: main.kode)
	BuildOutput string // output name without extension (default: main)
	Bytecode    bool   // emit .kbc bytecode (default: true)
	TargetDir   string // output directory (default: target)

	// [run]
	RunEntry string // source file to run (default: main.kode)
}

// defaultProjectConfig returns a ProjectConfig with sane defaults.
func defaultProjectConfig() ProjectConfig {
	return ProjectConfig{
		BuildEntry:  "main.kode",
		BuildOutput: "main",
		Bytecode:    true,
		TargetDir:   "target",
		RunEntry:    "main.kode",
	}
}

// LoadProjectConfig parses a kode.toml file and returns a ProjectConfig.
// Unrecognised keys are silently ignored.
// Returns nil when the file cannot be opened.
func LoadProjectConfig(tomlPath string) *ProjectConfig {
	f, err := os.Open(tomlPath)
	if err != nil {
		return nil
	}
	defer f.Close()

	cfg := defaultProjectConfig()
	section := ""

	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Section header [name]
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.ToLower(strings.TrimSpace(line[1 : len(line)-1]))
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}
		key := strings.TrimSpace(parts[0])
		val := strings.TrimSpace(parts[1])
		// Strip surrounding quotes
		if len(val) >= 2 && val[0] == '"' && val[len(val)-1] == '"' {
			val = val[1 : len(val)-1]
		}

		switch section {
		case "project":
			switch key {
			case "name":
				cfg.Name = val
			case "version":
				cfg.Version = val
			case "edition":
				cfg.Edition = val
			case "description":
				cfg.Description = val
			case "kode_version":
				cfg.KodeVersion = val
			}
		case "build":
			switch key {
			case "entry":
				cfg.BuildEntry = val
			case "output":
				cfg.BuildOutput = val
			case "bytecode":
				cfg.Bytecode = val == "true"
			case "target_dir":
				cfg.TargetDir = val
			}
		case "run":
			switch key {
			case "entry":
				cfg.RunEntry = val
			}
		}
	}

	return &cfg
}

// FindProjectToml returns "kode.toml" if one exists in the current working
// directory, otherwise an empty string.
func FindProjectToml() string {
	if _, err := os.Stat("kode.toml"); err == nil {
		return "kode.toml"
	}
	return ""
}
