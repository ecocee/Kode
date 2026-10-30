package cli_test

import (
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestRunCommand(t *testing.T) {
	// build CLI binary
	exe := filepath.Join(os.TempDir(), "kode_test.exe")
	build := exec.Command("go", "build", "-o", exe, filepath.Join("..", "..", "cmd", "kode"))
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("failed to build CLI: %v output: %s", err, out)
	}
	defer os.Remove(exe)

	// create temporary source file
	src := "print(1 + 2)"
	tmp := filepath.Join(os.TempDir(), "test1.kode")
	ioutil.WriteFile(tmp, []byte(src), 0644)
	defer os.Remove(tmp)

	cmd := exec.Command(exe, "run", tmp)
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("run command failed: %v output: %s", err, out)
	}
	if string(out) != "3\n" {
		t.Fatalf("unexpected output: %s", out)
	}
}

func TestBuildCommandProducesBinary(t *testing.T) {
	// ensure CLI binary built
	exe := filepath.Join(os.TempDir(), "kode_test.exe")
	build := exec.Command("go", "build", "-o", exe, filepath.Join("..", "..", "cmd", "kode"))
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("failed to build CLI: %v output: %s", err, out)
	}
	defer os.Remove(exe)

	src := "print(42)"
	tmp := filepath.Join(os.TempDir(), "test2.kode")
	bin := filepath.Join(os.TempDir(), "test2.exe")
	ioutil.WriteFile(tmp, []byte(src), 0644)
	defer os.Remove(tmp)
	defer os.Remove(bin)

	cmd := exec.Command(exe, "build", tmp, "--verbose", "--output", bin)
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("build command failed: %v output: %s", err, out)
	}
	if _, err := os.Stat(bin); err != nil {
		t.Fatalf("expected binary %s, got error %v", bin, err)
	}
}

func TestBadSyntax(t *testing.T) {
	// build CLI binary
	exe := filepath.Join(os.TempDir(), "kode_test.exe")
	build := exec.Command("go", "build", "-o", exe, filepath.Join("..", "..", "cmd", "kode"))
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("failed to build CLI: %v output: %s", err, out)
	}
	defer os.Remove(exe)

	src := "let = 1"
	tmp := filepath.Join(os.TempDir(), "test3.kode")
	ioutil.WriteFile(tmp, []byte(src), 0644)
	defer os.Remove(tmp)

	cmd := exec.Command(exe, "run", tmp)
	err := cmd.Run()
	if err == nil {
		t.Fatalf("expected error for bad syntax")
	}
}
