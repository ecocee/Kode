package runtime_test

import (
	"bytes"
	"os"
	"testing"

	"github.com/ecocee/kode-go/internal/compiler"
	"github.com/ecocee/kode-go/internal/parser"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/runtime"
)

// captureStdout temporarily redirects os.Stdout and returns whatever was written.
func captureStdout(f func()) string {
	orig := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	f()

	w.Close()
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	os.Stdout = orig
	return buf.String()
}

func TestGlobalLetAndPrint(t *testing.T) {
	src := "let a = 1\nlet b = 2\nprint(a + b)"
	p, err := parser.NewParser("test.kode", src)
	if err != nil {
		t.Fatal(err)
	}
	stmts, err := p.Parse()
	if err != nil {
		t.Fatal(err)
	}
	c := compiler.NewCompiler()
	ir, err := c.Compile(ast.Program{Statements: stmts})
	if err != nil {
		t.Fatal(err)
	}
	rt := runtime.NewRuntime()
	out := captureStdout(func() {
		if err := rt.Execute(ir); err != nil {
			t.Fatal(err)
		}
	})
	if out != "3\n" {
		t.Fatalf("expected 3\n, got %q", out)
	}
}
