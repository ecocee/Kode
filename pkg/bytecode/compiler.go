package bytecode

import (
	"fmt"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// Compiler converts IR to bytecode
type Compiler struct {
	buf            *Buffer
	localVars      map[string]int // Local variable name -> stack offset
	globalVars     map[string]int // Global variable name -> index
	functions      map[string]*ir.IRFunction
	astFunctions   map[string]*ast.FunctionDefStmt // AST functions for inlining
	currentFunc    *ir.IRFunction
	isInFunction   bool
	localVarOffset int
}

// NewCompiler creates a new bytecode compiler
func NewCompiler() *Compiler {
	return &Compiler{
		buf:          NewBuffer(),
		localVars:    make(map[string]int),
		globalVars:   make(map[string]int),
		functions:    make(map[string]*ir.IRFunction),
		astFunctions: make(map[string]*ast.FunctionDefStmt),
	}
}

// Compile converts IR to bytecode program
func (c *Compiler) Compile(ir *ir.IR) (*Program, error) {
	// Index functions first
	if ir.Program != nil {
		for _, fn := range ir.Program.Functions {
			c.functions[fn.Name] = fn
		}
	}

	// Collect AST function definitions for inlining
	for _, stmt := range ir.AST.Statements {
		if fnStmt, ok := stmt.(ast.FunctionDefStmt); ok {
			c.astFunctions[fnStmt.Name] = &fnStmt
		}
	}

	// Compile global statements
	for _, stmt := range ir.AST.Statements {
		if err := c.compileStatement(stmt); err != nil {
			return nil, err
		}
	}

	// Add halt instruction
	c.buf.Emit(OpHalt)

	return c.buf.Build(), nil
}

// compileStatement compiles a statement to bytecode
func (c *Compiler) compileStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.LetStmt:
		return c.compileLetStatement(&s)
	case ast.ExprStmt:
		return c.compileExpressionStatement(&s)
	case ast.AssignStmt:
		return c.compileAssignStatement(&s)
	case ast.ReturnStmt:
		return c.compileReturnStatement(&s)
	case ast.IfStmt:
		return c.compileIfStatement(&s)
	case ast.ForStmt:
		return c.compileForLoop(&s)
	case ast.WhileStmt:
		return c.compileWhileLoop(&s)
	case ast.FunctionDefStmt:
		return c.compileFunctionDeclaration(&s)
	case ast.BreakStmt:
		c.buf.Emit(OpBreak)
		return nil
	case ast.ContinueStmt:
		c.buf.Emit(OpContinue)
		return nil
	case ast.StructDeclStmt:
		// Structs are type definitions, no runtime code needed
		return nil
	case ast.EnumDeclStmt:
		// Enums are type definitions, no runtime code needed
		return nil
	default:
		return fmt.Errorf("unknown statement type: %T", stmt)
	}
}

// compileAssignStatement compiles an assignment statement
func (c *Compiler) compileAssignStatement(stmt *ast.AssignStmt) error {
	// Compile the value expression
	if err := c.compileExpression(stmt.Value); err != nil {
		return err
	}

	// Store to variable
	varIdx := c.buf.AddGlobal(stmt.Name)
	c.buf.Emit(OpStoreGlobal, varIdx)

	return nil
}

// compileLetStatement compiles a let statement
func (c *Compiler) compileLetStatement(stmt *ast.LetStmt) error {
	// Compile the value expression
	if err := c.compileExpression(stmt.Value); err != nil {
		return err
	}

	// Store to variable
	varIdx := c.buf.AddGlobal(stmt.Name)
	c.buf.Emit(OpStoreGlobal, varIdx)

	return nil
}

// compileExpressionStatement compiles an expression statement
func (c *Compiler) compileExpressionStatement(stmt *ast.ExprStmt) error {
	if err := c.compileExpression(stmt.Expr); err != nil {
		return err
	}
	// Pop result
	c.buf.Emit(OpPop)
	return nil
}

// compileReturnStatement compiles a return statement
func (c *Compiler) compileReturnStatement(stmt *ast.ReturnStmt) error {
	if stmt.Value != nil {
		if err := c.compileExpression(stmt.Value); err != nil {
			return err
		}
		c.buf.Emit(OpReturnValue)
	} else {
		c.buf.Emit(OpReturn)
	}
	return nil
}

// compileIfStatement compiles an if statement
func (c *Compiler) compileIfStatement(stmt *ast.IfStmt) error {
	// Evaluate condition
	if err := c.compileExpression(stmt.Condition); err != nil {
		return err
	}

	// Emit jump if false (placeholder - will be patched)
	jmpIfFalseIdx := len(c.buf.instructions)
	c.buf.Emit(OpJmpIfFalse, 0) // Placeholder offset

	// Compile then branch
	for _, s := range stmt.ThenBranch {
		if err := c.compileStatement(s); err != nil {
			return err
		}
	}

	// If there's an else branch, emit jump over it
	var jmpIdx int
	if len(stmt.ElseBranch) > 0 {
		jmpIdx = len(c.buf.instructions)
		c.buf.Emit(OpJmp, 0) // Placeholder offset
	}

	// Patch the JmpIfFalse to jump to else (or end if no else)
	falseOffset := len(c.buf.instructions) - jmpIfFalseIdx
	c.buf.instructions[jmpIfFalseIdx].Args[0] = falseOffset

	// Compile else branch if present
	if len(stmt.ElseBranch) > 0 {
		for _, s := range stmt.ElseBranch {
			if err := c.compileStatement(s); err != nil {
				return err
			}
		}
		// Patch the Jmp to jump to end
		jmpOffset := len(c.buf.instructions) - jmpIdx
		c.buf.instructions[jmpIdx].Args[0] = jmpOffset
	}

	return nil
}

// compileForLoop compiles a for loop
func (c *Compiler) compileForLoop(stmt *ast.ForStmt) error {
	// Compile initialization
	if stmt.Init != nil {
		switch init := stmt.Init.(type) {
		case ast.LetStmt:
			if err := c.compileLetStatement(&init); err != nil {
				return err
			}
		case ast.Statement:
			if err := c.compileStatement(init); err != nil {
				return err
			}
		}
	}

	// Save loop start position for jumping back
	loopStart := len(c.buf.instructions)

	// Compile condition
	if stmt.Condition != nil {
		if err := c.compileExpression(stmt.Condition); err != nil {
			return err
		}
		// Jump out if condition is false
		jmpIfFalseIdx := len(c.buf.instructions)
		c.buf.Emit(OpJmpIfFalse, 0) // Placeholder - will be patched

		// Compile loop body
		for _, s := range stmt.Body {
			if err := c.compileStatement(s); err != nil {
				return err
			}
		}

		// Compile increment
		if stmt.Incr != nil {
			// Try to compile as expression, or handle as assignment
			if binExpr, ok := stmt.Incr.(ast.BinaryExpr); ok && binExpr.Op == ast.OpAssign {
				// This is an assignment-like binary expression (e.g., i = i + 1)
				// Evaluate the right side
				if err := c.compileExpression(binExpr.Right); err != nil {
					return err
				}
				// Store to variable
				if ident, ok := binExpr.Left.(ast.IdentifierExpr); ok {
					varIdx := c.buf.AddGlobal(ident.Name)
					c.buf.Emit(OpStoreGlobal, varIdx)
				}
			} else {
				if err := c.compileExpression(stmt.Incr); err != nil {
					return err
				}
				c.buf.Emit(OpPop) // Pop the result
			}
		}

		// Jump back to loop condition
		jmpOffset := loopStart - len(c.buf.instructions) - 1
		c.buf.Emit(OpJmp, jmpOffset)

		// Patch the JmpIfFalse to exit the loop
		falseOffset := len(c.buf.instructions) - jmpIfFalseIdx
		c.buf.instructions[jmpIfFalseIdx].Args[0] = falseOffset
	} else {
		// No condition - compile body and jump back (infinite loop)
		for _, s := range stmt.Body {
			if err := c.compileStatement(s); err != nil {
				return err
			}
		}

		// Compile increment
		if stmt.Incr != nil {
			// Try to compile as expression, or handle as assignment
			if binExpr, ok := stmt.Incr.(ast.BinaryExpr); ok && binExpr.Op == ast.OpAssign {
				// This is an assignment-like binary expression (e.g., i = i + 1)
				// Evaluate the right side
				if err := c.compileExpression(binExpr.Right); err != nil {
					return err
				}
				// Store to variable
				if ident, ok := binExpr.Left.(ast.IdentifierExpr); ok {
					varIdx := c.buf.AddGlobal(ident.Name)
					c.buf.Emit(OpStoreGlobal, varIdx)
				}
			} else {
				if err := c.compileExpression(stmt.Incr); err != nil {
					return err
				}
				c.buf.Emit(OpPop)
			}
		}

		// Jump back to loop start
		jmpOffset := loopStart - len(c.buf.instructions) - 1
		c.buf.Emit(OpJmp, jmpOffset)
	}

	return nil
}

// compileWhileLoop compiles a while loop
func (c *Compiler) compileWhileLoop(stmt *ast.WhileStmt) error {
	// Record the start of the loop
	loopStart := len(c.buf.instructions)

	// Compile condition
	if stmt.Condition != nil {
		if err := c.compileExpression(stmt.Condition); err != nil {
			return err
		}
		// Emit jump if false (placeholder offset)
		c.buf.Emit(OpJmpIfFalse, 0)
		jmpIfFalseIdx := len(c.buf.instructions) - 1

		// Compile loop body
		for _, s := range stmt.Body {
			if err := c.compileStatement(s); err != nil {
				return err
			}
		}

		// Jump back to loop start
		jmpOffset := loopStart - len(c.buf.instructions) - 1
		c.buf.Emit(OpJmp, jmpOffset)

		// Patch the JmpIfFalse to exit the loop
		falseOffset := len(c.buf.instructions) - jmpIfFalseIdx
		c.buf.instructions[jmpIfFalseIdx].Args[0] = falseOffset
	} else {
		// No condition - infinite loop
		for _, s := range stmt.Body {
			if err := c.compileStatement(s); err != nil {
				return err
			}
		}
		// Jump back to loop start
		jmpOffset := loopStart - len(c.buf.instructions) - 1
		c.buf.Emit(OpJmp, jmpOffset)
	}
	return nil
}

// compileFunctionDeclaration compiles a function declaration
func (c *Compiler) compileFunctionDeclaration(stmt *ast.FunctionDefStmt) error {
	// Store function reference
	return nil
}

// compileExpression compiles an expression to bytecode
func (c *Compiler) compileExpression(expr ast.Expression) error {
	switch e := expr.(type) {
	case ast.NumberExpr:
		idx := c.buf.AddConstant(int(e.Value))
		c.buf.Emit(OpPush, idx)
	case ast.FloatExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.BoolExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.StringExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.IdentifierExpr:
		varIdx := c.buf.AddGlobal(e.Name)
		c.buf.Emit(OpLoadGlobal, varIdx)
	case ast.BinaryExpr:
		return c.compileBinaryExpression(&e)
	case ast.UnaryExpr:
		return c.compileUnaryExpression(&e)
	case ast.CallExpr:
		return c.compileCallExpression(&e)
	case ast.ArrayExpr:
		return c.compileArrayExpr(&e)
	case ast.ArrayAccessExpr:
		return c.compileArrayAccessExpr(&e)
	case ast.MemberAccessExpr:
		return c.compileMemberAccessExpr(&e)
	case ast.StructLiteralExpr:
		return c.compileStructLiteralExpr(&e)
	case ast.EnumVariantExpr:
		return c.compileEnumVariantExpr(&e)
	default:
		return fmt.Errorf("unknown expression type: %T", expr)
	}
	return nil
}

// compileBinaryExpression compiles a binary expression
func (c *Compiler) compileBinaryExpression(expr *ast.BinaryExpr) error {
	if err := c.compileExpression(expr.Left); err != nil {
		return err
	}
	if err := c.compileExpression(expr.Right); err != nil {
		return err
	}

	switch expr.Op {
	case ast.OpAdd:
		c.buf.Emit(OpAdd)
	case ast.OpSubtract:
		c.buf.Emit(OpSub)
	case ast.OpMultiply:
		c.buf.Emit(OpMul)
	case ast.OpDivide:
		c.buf.Emit(OpDiv)
	case ast.OpModulo:
		c.buf.Emit(OpMod)
	case ast.OpEqual:
		c.buf.Emit(OpEq)
	case ast.OpNotEqual:
		c.buf.Emit(OpNe)
	case ast.OpLessThan:
		c.buf.Emit(OpLt)
	case ast.OpGreaterThan:
		c.buf.Emit(OpGt)
	case ast.OpLessThanOrEqual:
		c.buf.Emit(OpLte)
	case ast.OpGreaterThanOrEqual:
		c.buf.Emit(OpGte)
	case ast.OpAnd:
		c.buf.Emit(OpAnd)
	case ast.OpOr:
		c.buf.Emit(OpOr)
	case ast.OpBitAnd:
		c.buf.Emit(OpBitAnd)
	case ast.OpBitOr:
		c.buf.Emit(OpBitOr)
	case ast.OpBitXor:
		c.buf.Emit(OpBitXor)
	case ast.OpBitShl:
		c.buf.Emit(OpBitShl)
	case ast.OpBitShr:
		c.buf.Emit(OpBitShr)
	default:
		return fmt.Errorf("unknown operator: %v", expr.Op)
	}
	return nil
}

// compileUnaryExpression compiles a unary expression
func (c *Compiler) compileUnaryExpression(expr *ast.UnaryExpr) error {
	switch expr.Op {
	case ast.OpNegate:
		if err := c.compileExpression(expr.Expr); err != nil {
			return err
		}
		c.buf.Emit(OpNeg)

	case ast.OpNot:
		if err := c.compileExpression(expr.Expr); err != nil {
			return err
		}
		c.buf.Emit(OpNot)

	case ast.OpBitNot:
		if err := c.compileExpression(expr.Expr); err != nil {
			return err
		}
		c.buf.Emit(OpBitNot)

	case ast.OpPostInc:
		// Handle postfix increment: i++
		// Returns old value but increments variable
		if ident, ok := expr.Expr.(ast.IdentifierExpr); ok {
			varIdx := c.buf.AddGlobal(ident.Name)
			// Load current value
			c.buf.Emit(OpLoadGlobal, varIdx)
			// Duplicate to keep old value
			c.buf.Emit(OpDup)
			// Increment the copy
			c.buf.Emit(OpIncr)
			// Store the incremented value back
			c.buf.Emit(OpStoreGlobal, varIdx)
			// Remove incremented value from stack, leaving old value
			c.buf.Emit(OpPop)
		} else {
			if err := c.compileExpression(expr.Expr); err != nil {
				return err
			}
			c.buf.Emit(OpIncr)
		}

	case ast.OpPostDec:
		// Handle postfix decrement: i--
		// Returns old value but decrements variable
		if ident, ok := expr.Expr.(ast.IdentifierExpr); ok {
			varIdx := c.buf.AddGlobal(ident.Name)
			// Load current value
			c.buf.Emit(OpLoadGlobal, varIdx)
			// Duplicate to keep old value
			c.buf.Emit(OpDup)
			// Decrement the copy
			c.buf.Emit(OpDecr)
			// Store the decremented value back
			c.buf.Emit(OpStoreGlobal, varIdx)
			// Remove decremented value from stack, leaving old value
			c.buf.Emit(OpPop)
		} else {
			if err := c.compileExpression(expr.Expr); err != nil {
				return err
			}
			c.buf.Emit(OpDecr)
		}

	default:
		return fmt.Errorf("unknown unary operator: %v", expr.Op)
	}
	return nil
}

// compileCallExpression compiles a function call
func (c *Compiler) compileCallExpression(expr *ast.CallExpr) error {
	// Get function name
	if ident, ok := expr.Callee.(ast.IdentifierExpr); ok {
		// Handle built-in print function specially
		if ident.Name == "print" {
			for _, arg := range expr.Arguments {
				if err := c.compileExpression(arg); err != nil {
					return err
				}
				c.buf.Emit(OpPrint)
			}
			// Push nil as return value
			idx := c.buf.AddConstant(nil)
			c.buf.Emit(OpPush, idx)
			return nil
		}
		if ident.Name == "input" {
			// Handle the prompt argument if provided
			if len(expr.Arguments) > 0 {
				if err := c.compileExpression(expr.Arguments[0]); err != nil {
					return err
				}
				c.buf.Emit(OpInputWithPrompt)
			} else {
				c.buf.Emit(OpInput)
			}
			return nil
		}

		// Try to inline user-defined functions
		if fnDef, ok := c.astFunctions[ident.Name]; ok {
			// For expression-bodied functions, inline the body
			if fnDef.IsExprBody {
				// Create a mapping of parameter names to argument expressions
				paramMap := make(map[string]ast.Expression)
				for i, param := range fnDef.Params {
					if i < len(expr.Arguments) {
						paramMap[param.Name] = expr.Arguments[i]
					}
				}

				// Compile the function body with parameter substitution
				bodyExpr := fnDef.Body.(ast.Expression)
				return c.compileExpressionWithSubstitution(bodyExpr, paramMap)
			}
		}

		// Compile arguments
		for _, arg := range expr.Arguments {
			if err := c.compileExpression(arg); err != nil {
				return err
			}
		}

		c.buf.Emit(OpCall, ident.Name, len(expr.Arguments))
	}

	return nil
}

// compileExpressionWithSubstitution compiles an expression with parameter substitution
func (c *Compiler) compileExpressionWithSubstitution(expr ast.Expression, paramMap map[string]ast.Expression) error {
	switch e := expr.(type) {
	case ast.NumberExpr:
		idx := c.buf.AddConstant(int(e.Value))
		c.buf.Emit(OpPush, idx)
	case ast.FloatExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.BoolExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.StringExpr:
		idx := c.buf.AddConstant(e.Value)
		c.buf.Emit(OpPush, idx)
	case ast.IdentifierExpr:
		// Check if this is a parameter
		if argExpr, ok := paramMap[e.Name]; ok {
			// Substitute the parameter with the argument expression
			return c.compileExpressionWithSubstitution(argExpr, paramMap)
		}
		// Otherwise, load as a variable
		varIdx := c.buf.AddGlobal(e.Name)
		c.buf.Emit(OpLoadGlobal, varIdx)
	case ast.BinaryExpr:
		// Compile left and right with substitution
		if err := c.compileExpressionWithSubstitution(e.Left, paramMap); err != nil {
			return err
		}
		if err := c.compileExpressionWithSubstitution(e.Right, paramMap); err != nil {
			return err
		}

		switch e.Op {
		case ast.OpAdd:
			c.buf.Emit(OpAdd)
		case ast.OpSubtract:
			c.buf.Emit(OpSub)
		case ast.OpMultiply:
			c.buf.Emit(OpMul)
		case ast.OpDivide:
			c.buf.Emit(OpDiv)
		case ast.OpModulo:
			c.buf.Emit(OpMod)
		case ast.OpEqual:
			c.buf.Emit(OpEq)
		case ast.OpNotEqual:
			c.buf.Emit(OpNe)
		case ast.OpLessThan:
			c.buf.Emit(OpLt)
		case ast.OpLessThanOrEqual:
			c.buf.Emit(OpLte)
		case ast.OpGreaterThan:
			c.buf.Emit(OpGt)
		case ast.OpGreaterThanOrEqual:
			c.buf.Emit(OpGte)
		case ast.OpAnd:
			c.buf.Emit(OpAnd)
		case ast.OpOr:
			c.buf.Emit(OpOr)
		case ast.OpBitAnd:
			c.buf.Emit(OpBitAnd)
		case ast.OpBitOr:
			c.buf.Emit(OpBitOr)
		case ast.OpBitXor:
			c.buf.Emit(OpBitXor)
		case ast.OpBitShl:
			c.buf.Emit(OpBitShl)
		case ast.OpBitShr:
			c.buf.Emit(OpBitShr)
		default:
			return fmt.Errorf("unknown binary operator: %v", e.Op)
		}
	case ast.UnaryExpr:
		if err := c.compileExpressionWithSubstitution(e.Expr, paramMap); err != nil {
			return err
		}

		switch e.Op {
		case ast.OpNegate:
			c.buf.Emit(OpNeg)
		case ast.OpNot:
			c.buf.Emit(OpNot)
		default:
			return fmt.Errorf("unknown unary operator: %v", e.Op)
		}
	case ast.CallExpr:
		// Handle recursive calls or nested calls
		if callIdent, ok := e.Callee.(ast.IdentifierExpr); ok {
			if fnDef, ok := c.astFunctions[callIdent.Name]; ok {
				if fnDef.IsExprBody {
					// Create parameter map for the nested call
					nestedParamMap := make(map[string]ast.Expression)
					for i, param := range fnDef.Params {
						if i < len(e.Arguments) {
							nestedParamMap[param.Name] = e.Arguments[i]
						}
					}
					bodyExpr := fnDef.Body.(ast.Expression)
					return c.compileExpressionWithSubstitution(bodyExpr, nestedParamMap)
				}
			}
		}
		// Fall back to regular expression compilation
		return c.compileExpression(e)
	default:
		return fmt.Errorf("unknown expression type: %T", expr)
	}
	return nil
}

// compileArrayExpr compiles array literal: [1, 2, 3]
func (c *Compiler) compileArrayExpr(expr *ast.ArrayExpr) error {
	// Compile each element onto the stack
	for _, elem := range expr.Elements {
		if err := c.compileExpression(elem); err != nil {
			return err
		}
	}
	// Emit instruction to create array with element count
	c.buf.Emit(OpArrayCreate, len(expr.Elements))
	return nil
}

// compileArrayAccessExpr compiles array access: arr[index]
func (c *Compiler) compileArrayAccessExpr(expr *ast.ArrayAccessExpr) error {
	// Compile array expression
	if err := c.compileExpression(expr.Array); err != nil {
		return err
	}
	// Compile index expression
	if err := c.compileExpression(expr.Index); err != nil {
		return err
	}
	// Emit array access instruction
	c.buf.Emit(OpArrayAccess)
	return nil
}

// compileMemberAccessExpr compiles member access: obj.member
func (c *Compiler) compileMemberAccessExpr(expr *ast.MemberAccessExpr) error {
	// Compile object expression
	if err := c.compileExpression(expr.Object); err != nil {
		return err
	}
	// Emit member access instruction
	memberIdx := c.buf.AddConstant(expr.Member)
	c.buf.Emit(OpMemberAccess, memberIdx)
	return nil
}

// compileStructLiteralExpr compiles struct literal: StructName { field1: value1, ... }
func (c *Compiler) compileStructLiteralExpr(expr *ast.StructLiteralExpr) error {
	// For each field in the struct, compile the value and push to stack
	for fieldName, fieldVal := range expr.Fields {
		if err := c.compileExpression(fieldVal); err != nil {
			return err
		}
		// Add field name as constant
		fieldIdx := c.buf.AddConstant(fieldName)
		c.buf.Emit(OpPush, fieldIdx)
	}
	
	// Emit struct create instruction
	structIdx := c.buf.AddConstant(expr.StructName)
	fieldCount := len(expr.Fields)
	c.buf.Emit(OpStructCreate, structIdx, fieldCount)
	return nil
}
// compileEnumVariantExpr compiles enum variant: EnumName::Variant(value)
func (c *Compiler) compileEnumVariantExpr(expr *ast.EnumVariantExpr) error {
	// Compile the enum variant value if provided
	if expr.Value != nil {
		if err := c.compileExpression(expr.Value); err != nil {
			return err
		}
	}
	
	// Emit enum variant instruction
	enumIdx := c.buf.AddConstant(expr.EnumName)
	variantIdx := c.buf.AddConstant(expr.Variant)
	c.buf.Emit(OpEnumVariant, enumIdx, variantIdx)
	return nil
}