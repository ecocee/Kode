package compiler

import (
	"fmt"

	"github.com/ecocee/kode-go/internal/typer"
	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// Compiler compiles AST to IR
type Compiler struct {
	ir      *ir.IR
	typer   *typer.Typer
	nextVar int
}

// NewCompiler creates a new compiler
func NewCompiler() *Compiler {
	return &Compiler{
		ir:      ir.NewIR(),
		typer:   typer.NewTyper(),
		nextVar: 0,
	}
}

// Compile compiles a program to IR
func (c *Compiler) Compile(program ast.Program) (*ir.IR, error) {
	if err := c.typer.CheckProgram(program); err != nil {
		return nil, err
	}

	for _, stmt := range program.Statements {
		if err := c.compileStatement(stmt); err != nil {
			return nil, err
		}
	}

	return c.ir, nil
}

// compileStatement compiles a statement
func (c *Compiler) compileStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.FunctionDefStmt:
		fn := &ir.IRFunction{
			Name:       s.Name,
			Params:     make([]string, len(s.Params)),
			ReturnType: s.ReturnType,
			Body:       []*ir.IRBlock{},
		}
		for i, param := range s.Params {
			fn.Params[i] = param.Name
		}
		c.ir.Program.Functions = append(c.ir.Program.Functions, fn)
		// Compile body (simplified)
		block := &ir.IRBlock{Label: "entry", Instructions: []ir.IRInstruction{}}
		if s.IsExprBody {
			// Body is an Expression - compile as return
			value := c.compileExpression(s.Body.(ast.Expression))
			block.Instructions = append(block.Instructions, ir.IRReturn{Value: value})
		} else {
			// Body is []Statement
			for _, bodyStmt := range s.Body.([]ast.Statement) {
				if err := c.compileStatementToBlock(bodyStmt, block); err != nil {
					return err
				}
			}
		}
		fn.Body = append(fn.Body, block)
	case ast.LetStmt:
		// Add to globals or locals (simplified)
		global := &ir.IRGlobal{
			Name:  s.Name,
			Type:  s.Type,
			Value: c.compileExpression(s.Value),
		}
		c.ir.Program.Globals = append(c.ir.Program.Globals, global)
	}
	return nil
}

// compileStatementToBlock compiles a statement to a block
func (c *Compiler) compileStatementToBlock(stmt ast.Statement, block *ir.IRBlock) error {
	switch s := stmt.(type) {
	case ast.ReturnStmt:
		val := c.compileExpression(s.Value)
		block.Instructions = append(block.Instructions, ir.IRReturn{Value: val})
	case ast.ExprStmt:
		c.compileExpression(s.Expr) // Ignore result
	}
	return nil
}

// compileExpression compiles an expression
func (c *Compiler) compileExpression(expr ast.Expression) ir.IRValue {
	switch e := expr.(type) {
	case ast.NumberExpr:
		return ir.IRConstant{Type: ast.IntType{}, Value: e.Value}
	case ast.IdentifierExpr:
		return ir.IRVariable{Name: e.Name, Type: ast.IntType{}} // Simplified
	case ast.BinaryExpr:
		left := c.compileExpression(e.Left)
		right := c.compileExpression(e.Right)
		result := c.newVar()
		block := c.currentBlock()
		block.Instructions = append(block.Instructions, ir.IRBinaryOp{
			Op:     e.Op,
			Left:   left,
			Right:  right,
			Result: result,
		})
		return ir.IRVariable{Name: result, Type: ast.IntType{}}
	}
	return ir.IRConstant{Type: ast.IntType{}, Value: 0}
}

// newVar generates a new variable name
func (c *Compiler) newVar() string {
	c.nextVar++
	return fmt.Sprintf("v%d", c.nextVar)
}

// currentBlock gets the current block (simplified)
func (c *Compiler) currentBlock() *ir.IRBlock {
	if len(c.ir.Program.Functions) > 0 {
		fn := c.ir.Program.Functions[len(c.ir.Program.Functions)-1]
		if len(fn.Body) > 0 {
			return fn.Body[len(fn.Body)-1]
		}
	}
	return nil
}
