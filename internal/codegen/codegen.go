package codegen

import (
	"fmt"
	"strings"

	"github.com/ecocee/kode-go/pkg/ir"
)

// CodeGenerator generates Go code from IR
type CodeGenerator struct {
	buf strings.Builder
}

// NewCodeGenerator creates a new code generator
func NewCodeGenerator() *CodeGenerator {
	return &CodeGenerator{}
}

// Generate generates Go code from IR
func (g *CodeGenerator) Generate(irProgram *ir.IR) (string, error) {
	g.buf.Reset()

	// Write package and imports
	g.buf.WriteString("package main\n\n")
	g.buf.WriteString("import (\n")
	g.buf.WriteString("\t\"fmt\"\n")
	g.buf.WriteString(")\n\n")

	// Write globals
	if len(irProgram.Program.Globals) > 0 {
		g.buf.WriteString("var (\n")
		for _, global := range irProgram.Program.Globals {
			g.buf.WriteString(fmt.Sprintf("\t%s interface{}\n", global.Name))
		}
		g.buf.WriteString(")\n\n")
	}

	// Write functions
	for _, fn := range irProgram.Program.Functions {
		g.generateFunction(fn)
	}

	// Always add a main function
	g.buf.WriteString("func main() {\n")
	// Call the user-defined main if it exists
	hasUserMain := false
	for _, fn := range irProgram.Program.Functions {
		if fn.Name == "main" {
			hasUserMain = true
			break
		}
	}
	if !hasUserMain {
		// If no main function defined by user, just exit
		g.buf.WriteString("\t// No main function\n")
	}
	g.buf.WriteString("}\n")

	return g.buf.String(), nil
}

// generateFunction generates a function
func (g *CodeGenerator) generateFunction(fn *ir.IRFunction) {
	g.buf.WriteString(fmt.Sprintf("func %s(", fn.Name))
	for i, param := range fn.Params {
		if i > 0 {
			g.buf.WriteString(", ")
		}
		g.buf.WriteString(fmt.Sprintf("%s interface{}", param))
	}
	g.buf.WriteString(") interface{} {\n")

	// Write function body
	for _, block := range fn.Body {
		g.generateBlock(block)
	}

	g.buf.WriteString("\treturn nil\n")
	g.buf.WriteString("}\n\n")
}

// generateBlock generates a block
func (g *CodeGenerator) generateBlock(block *ir.IRBlock) {
	for _, instr := range block.Instructions {
		g.generateInstruction(instr)
	}
}

// generateInstruction generates an instruction
func (g *CodeGenerator) generateInstruction(instr ir.IRInstruction) {
	switch i := instr.(type) {
	case ir.IRCall:
		g.generateCall(&i)
	case ir.IRBinaryOp:
		g.generateBinaryOp(&i)
	case ir.IRReturn:
		g.generateReturn(&i)
	}
}

// generateCall generates a function call
func (g *CodeGenerator) generateCall(call *ir.IRCall) {
	result := call.Result
	args := make([]string, len(call.Args))
	for i, arg := range call.Args {
		args[i] = g.valueToString(arg)
	}

	switch call.Function {
	case "print":
		if len(args) > 0 {
			g.buf.WriteString(fmt.Sprintf("\tfmt.Println(%s)\n", args[0]))
		}
	default:
		g.buf.WriteString(fmt.Sprintf("\t%s := %s(%s)\n", result, call.Function, strings.Join(args, ", ")))
	}
}

// generateBinaryOp generates a binary operation
func (g *CodeGenerator) generateBinaryOp(op *ir.IRBinaryOp) {
	left := g.valueToString(op.Left)
	right := g.valueToString(op.Right)

	var goOp string
	switch op.Op {
	case 0: // OpAdd
		goOp = "+"
	case 1: // OpSubtract
		goOp = "-"
	case 2: // OpMultiply
		goOp = "*"
	case 3: // OpDivide
		goOp = "/"
	default:
		goOp = "+"
	}

	g.buf.WriteString(fmt.Sprintf("\t%s := %s %s %s\n", op.Result, left, goOp, right))
}

// generateReturn generates a return statement
func (g *CodeGenerator) generateReturn(ret *ir.IRReturn) {
	val := g.valueToString(ret.Value)
	g.buf.WriteString(fmt.Sprintf("\treturn %s\n", val))
}

// valueToString converts IR value to Go code
func (g *CodeGenerator) valueToString(val ir.IRValue) string {
	switch v := val.(type) {
	case ir.IRConstant:
		switch val := v.Value.(type) {
		case string:
			return fmt.Sprintf("%q", val)
		case int64:
			return fmt.Sprintf("%d", val)
		case float64:
			return fmt.Sprintf("%f", val)
		case bool:
			return fmt.Sprintf("%v", val)
		default:
			return "nil"
		}
	case ir.IRVariable:
		return v.Name
	default:
		return "nil"
	}
}
