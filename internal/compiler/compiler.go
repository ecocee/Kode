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
	// store AST for runtime fallback
	c.ir.AST = program

	// First pass: collect all function definitions
	for _, stmt := range program.Statements {
		if fnStmt, ok := stmt.(ast.FunctionDefStmt); ok {
			if err := c.compileStatement(fnStmt); err != nil {
				return nil, err
			}
		}
	}

	// Create an implicit entry point for top-level statements
	entryBlock := &ir.IRBlock{Label: "entry", Instructions: []ir.IRInstruction{}}
	hasTopLevelStatements := false

	// Second pass: compile all statements (including non-function statements)
	for _, stmt := range program.Statements {
		switch stmt.(type) {
		case ast.FunctionDefStmt:
			// Already compiled in first pass
		case ast.LetStmt, ast.AssignStmt:
			// Variable declarations at top level
			if err := c.compileStatement(stmt); err != nil {
				return nil, err
			}
		default:
			// Other top-level statements go into the entry block
			hasTopLevelStatements = true
			if err := c.compileStatementToBlock(stmt, entryBlock); err != nil {
				return nil, err
			}
		}
	}

	// If there are top-level statements, create a main function for them
	if hasTopLevelStatements && len(entryBlock.Instructions) > 0 {
		// Check if main already exists
		mainExists := false
		var mainFn *ir.IRFunction
		for i, fn := range c.ir.Program.Functions {
			if fn.Name == "main" {
				mainExists = true
				mainFn = c.ir.Program.Functions[i]
				break
			}
		}

		if !mainExists {
			newMainFn := &ir.IRFunction{
				Name:       "main",
				Params:     []string{},
				ReturnType: ast.VoidType{},
				Body:       []*ir.IRBlock{entryBlock},
			}
			c.ir.Program.Functions = append(c.ir.Program.Functions, newMainFn)
		} else {
			// Merge top-level statements with existing main function
			// Add top-level instructions at the beginning of the first block
			if len(mainFn.Body) > 0 {
				mainFn.Body[0].Instructions = append(entryBlock.Instructions, mainFn.Body[0].Instructions...)
			} else {
				mainFn.Body = []*ir.IRBlock{entryBlock}
			}
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
	case ast.ExprStmt:
		c.compileExpression(s.Expr)
	}
	return nil
}

// compileStatementToBlock compiles a statement to a block
func (c *Compiler) compileStatementToBlock(stmt ast.Statement, block *ir.IRBlock) error {
	switch s := stmt.(type) {
	case ast.ReturnStmt:
		val := c.compileExpressionWithBlock(s.Value, block)
		block.Instructions = append(block.Instructions, ir.IRReturn{Value: val})
	case ast.ExprStmt:
		c.compileExpressionWithBlock(s.Expr, block)
	}
	return nil
}

// compileExpressionWithBlock compiles an expression with a specific block context
func (c *Compiler) compileExpressionWithBlock(expr ast.Expression, block *ir.IRBlock) ir.IRValue {
	switch e := expr.(type) {
	case ast.NumberExpr:
		return ir.IRConstant{Type: ast.IntType{}, Value: e.Value}
	case ast.StringExpr:
		return ir.IRConstant{Type: ast.StringType{}, Value: e.Value}
	case ast.FloatExpr:
		return ir.IRConstant{Type: ast.FloatType{}, Value: e.Value}
	case ast.BoolExpr:
		return ir.IRConstant{Type: ast.BoolType{}, Value: e.Value}
	case ast.IdentifierExpr:
		return ir.IRVariable{Name: e.Name, Type: ast.IntType{}} // Simplified
	case ast.CallExpr:
		// Handle function calls
		fnName := ""
		if idExpr, ok := e.Callee.(ast.IdentifierExpr); ok {
			fnName = idExpr.Name
		}

		args := make([]ir.IRValue, len(e.Arguments))
		for i, arg := range e.Arguments {
			args[i] = c.compileExpressionWithBlock(arg, block)
		}

		result := c.newVar()
		call := ir.IRCall{
			Function: fnName,
			Args:     args,
			Result:   result,
		}

		// Add call instruction to the provided block
		if block != nil {
			block.Instructions = append(block.Instructions, call)
		}

		return ir.IRVariable{Name: result, Type: ast.VoidType{}}
	case ast.BinaryExpr:
		left := c.compileExpressionWithBlock(e.Left, block)
		right := c.compileExpressionWithBlock(e.Right, block)
		result := c.newVar()
		if block != nil {
			block.Instructions = append(block.Instructions, ir.IRBinaryOp{
				Op:     e.Op,
				Left:   left,
				Right:  right,
				Result: result,
			})
		}
		return ir.IRVariable{Name: result, Type: ast.IntType{}}
	case ast.UnaryExpr:
		operand := c.compileExpressionWithBlock(e.Expr, block)
		switch e.Op {
		case ast.OpPostInc, ast.OpPostDec:
			// For now, just return the operand since runtime handles it
			return operand
		}
		return operand
	}
	return ir.IRConstant{Type: ast.IntType{}, Value: 0}
}

// compileExpression compiles an expression
func (c *Compiler) compileExpression(expr ast.Expression) ir.IRValue {
	return c.compileExpressionWithBlock(expr, nil)
}

// newVar generates a new variable name
func (c *Compiler) newVar() string {
	c.nextVar++
	return fmt.Sprintf("v%d", c.nextVar)
}


