package bytecode

import (
	"fmt"
	"sort"

	"github.com/ecocee/kode-go/pkg/ast"
	"github.com/ecocee/kode-go/pkg/ir"
)

// LoopContext tracks loop start and break target positions
type LoopContext struct {
	startPC              int   // Position to jump to for continue
	breakTarget          int   // Position to jump to for break
	continueTarget       int   // Position to jump to for continue (start of increment)
	breakInstructions    []int // Instruction indices of OpBreak to patch
	continueInstructions []int // Instruction indices of OpContinue to patch
}

// FunctionInfo stores metadata about a function
type FunctionInfo struct {
	Name       string
	ParamCount int
	Params     []string
	IsExprBody bool
}

// Compiler converts IR to bytecode
type Compiler struct {
	buf            *Buffer
	localVars      map[string]int // Local variable name -> stack offset
	globalVars     map[string]int // Global variable name -> index
	functions      map[string]*ir.IRFunction
	astFunctions   map[string]*ast.FunctionDefStmt // AST functions for inlining
	functionInfo   map[string]*FunctionInfo        // Function metadata for code gen
	currentFunc    *ir.IRFunction
	currentFuncAST *ast.FunctionDefStmt // Current function being compiled
	isInFunction   bool
	localVarOffset int
	loopStack      []LoopContext // Stack of loop contexts for break/continue
	closureCounter int           // counter for generating unique closure names
}

// NewCompiler creates a new bytecode compiler
func NewCompiler() *Compiler {
	return &Compiler{
		buf:            NewBuffer(),
		localVars:      make(map[string]int),
		globalVars:     make(map[string]int),
		functions:      make(map[string]*ir.IRFunction),
		astFunctions:   make(map[string]*ast.FunctionDefStmt),
		functionInfo:   make(map[string]*FunctionInfo),
		loopStack:      make([]LoopContext, 0),
		closureCounter: 0,
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

	// Collect AST function definitions and their metadata
	for _, stmt := range ir.AST.Statements {
		if fnStmt, ok := stmt.(ast.FunctionDefStmt); ok {
			c.astFunctions[fnStmt.Name] = &fnStmt

			// Store function metadata
			paramNames := make([]string, len(fnStmt.Params))
			for i, param := range fnStmt.Params {
				paramNames[i] = param.Name
			}
			c.functionInfo[fnStmt.Name] = &FunctionInfo{
				Name:       fnStmt.Name,
				ParamCount: len(fnStmt.Params),
				Params:     paramNames,
				IsExprBody: fnStmt.IsExprBody,
			}
		}
	}

	// Emit skip jump to skip over all functions
	skipPC := len(c.buf.instructions)
	c.buf.Emit(OpJmp, 0) // Will patch later

	// Compile all function bodies first (while skipping them with the jump)
	for _, stmt := range ir.AST.Statements {
		if fnStmt, ok := stmt.(ast.FunctionDefStmt); ok {
			if err := c.compileFunctionBody(&fnStmt); err != nil {
				return nil, err
			}
		}
	}

	// Patch the skip jump to the start of main code
	mainCodePC := len(c.buf.instructions)
	c.buf.instructions[skipPC].Args[0] = mainCodePC - skipPC

	// Compile global statements (non-function declarations)
	for _, stmt := range ir.AST.Statements {
		if _, ok := stmt.(ast.FunctionDefStmt); !ok {
			if err := c.compileStatement(stmt); err != nil {
				return nil, err
			}
		}
	}

	// build resulting program
	return c.buf.Build(), nil
}

// compileStatement dispatches various statement types to their specific compilers
func (c *Compiler) compileStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.FunctionDefStmt:
		return c.compileFunctionDeclaration(&s)
	case ast.LetStmt:
		return c.compileLetStatement(&s)
	case ast.AssignStmt:
		return c.compileAssignStatement(&s)
	case ast.ConstDeclStmt:
		return c.compileConstStatement(&s)
	case ast.ExprStmt:
		return c.compileExpressionStatement(&s)
	case ast.ReturnStmt:
		return c.compileReturnStatement(&s)
	case ast.IfStmt:
		return c.compileIfStatement(&s)
	case ast.ForStmt:
		return c.compileForLoop(&s)
	case ast.ForInStmt:
		return c.compileForInLoop(&s)
	case ast.WhileStmt:
		return c.compileWhileLoop(&s)
	case ast.BreakStmt:
		if len(c.loopStack) == 0 {
			return fmt.Errorf("break statement outside of loop")
		}
		// Emit OpBreak with placeholder offset - will be patched later
		breakIdx := len(c.buf.instructions)
		c.buf.Emit(OpBreak, 0)
		c.loopStack[len(c.loopStack)-1].breakInstructions = append(c.loopStack[len(c.loopStack)-1].breakInstructions, breakIdx)
		return nil
	case ast.ContinueStmt:
		if len(c.loopStack) == 0 {
			return fmt.Errorf("continue statement outside of loop")
		}
		// Emit OpContinue with placeholder offset - will be patched later
		continueIdx := len(c.buf.instructions)
		c.buf.Emit(OpContinue, 0)
		c.loopStack[len(c.loopStack)-1].continueInstructions = append(c.loopStack[len(c.loopStack)-1].continueInstructions, continueIdx)
		return nil
	case ast.ImportStmt:
		return nil
	case ast.ExportStmt:
		return c.compileStatement(s.Statement)
	case ast.StructDeclStmt, ast.EnumDeclStmt:
		return nil
	case ast.TraitDeclStmt, ast.ImplDeclStmt, ast.ServiceDeclStmt, ast.ModuleDeclStmt:
		// Type/module declarations — no runtime code needed
		return nil
	case ast.HttpStmt, ast.RouteStmt:
		// HTTP declarations — not yet implemented
		return nil
	case ast.BlockStmt:
		// Compile inner block statements
		for _, inner := range s.Statements {
			if err := c.compileStatement(inner); err != nil {
				return err
			}
		}
		return nil
	case ast.PrintStmt:
		if err := c.compileExpression(s.Value); err != nil {
			return err
		}
		c.buf.Emit(OpPrint)
		return nil
	case ast.TryStmt:
		return c.compileTryStmt(&s)
	case ast.MatchStmt:
		return c.compileMatchStmt(&s)
	case ast.DeferStmt:
		return c.compileDeferStmt(&s)
	case ast.GoStmt, ast.SpawnStmt:
		// Not yet implemented — skip gracefully
		return nil
	default:
		return fmt.Errorf("unknown statement type: %T", stmt)
	}
}

func (c *Compiler) compileAssignStatement(stmt *ast.AssignStmt) error {
	// Compile the value expression
	if err := c.compileExpression(stmt.Value); err != nil {
		return err
	}

	// Store to local variable if inside a function, otherwise global
	if localIdx, ok := c.localVars[stmt.Name]; ok {
		c.buf.Emit(OpStore, localIdx)
	} else {
		varIdx := c.buf.AddGlobal(stmt.Name)
		c.buf.Emit(OpStoreGlobal, varIdx)
	}

	return nil
}

// compileLetStatement compiles a let statement
func (c *Compiler) compileLetStatement(stmt *ast.LetStmt) error {
	// Compile the value expression
	if err := c.compileExpression(stmt.Value); err != nil {
		return err
	}

	// If inside a function, allocate local variable; otherwise global
	if c.isInFunction {
		// Allocate new local variable
		varIdx := c.localVarOffset
		c.localVars[stmt.Name] = varIdx
		c.localVarOffset++
		c.buf.Emit(OpStore, varIdx)
	} else {
		// Store to global variable
		varIdx := c.buf.AddGlobal(stmt.Name)
		c.buf.Emit(OpStoreGlobal, varIdx)
	}

	return nil
}

// compileConstStatement compiles a const statement
func (c *Compiler) compileConstStatement(stmt *ast.ConstDeclStmt) error {
	// Compile the value expression
	if err := c.compileExpression(stmt.Value); err != nil {
		return err
	}

	// If inside a function, allocate local constant; otherwise global
	if c.isInFunction {
		// Allocate new local variable
		varIdx := c.localVarOffset
		c.localVars[stmt.Name] = varIdx
		c.localVarOffset++
		c.buf.Emit(OpStore, varIdx)
	} else {
		// Store to global variable (constants are stored like regular variables at runtime)
		varIdx := c.buf.AddGlobal(stmt.Name)
		c.buf.Emit(OpStoreGlobal, varIdx)
	}

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

	// Push loop context
	c.loopStack = append(c.loopStack, LoopContext{
		startPC:     loopStart,
		breakTarget: 0, // Will be set later
	})

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

		// Mark start of increment phase (where continue should jump)
		continueTargetPC := len(c.buf.instructions)

		// Compile increment
		if stmt.Incr != nil {
			// Try to compile as expression, or handle as assignment
			if binExpr, ok := stmt.Incr.(ast.BinaryExpr); ok && binExpr.Op == ast.OpAssign {
				// This is an assignment-like binary expression (e.g., i = i + 1)
				// Evaluate the right side
				if err := c.compileExpression(binExpr.Right); err != nil {
					return err
				}
				// Store to local or global variable
				if ident, ok := binExpr.Left.(ast.IdentifierExpr); ok {
					if localIdx, ok := c.localVars[ident.Name]; ok {
						c.buf.Emit(OpStore, localIdx)
					} else {
						varIdx := c.buf.AddGlobal(ident.Name)
						c.buf.Emit(OpStoreGlobal, varIdx)
					}
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

		// Update loop context with targets
		if len(c.loopStack) > 0 {
			c.loopStack[len(c.loopStack)-1].breakTarget = len(c.buf.instructions)
			c.loopStack[len(c.loopStack)-1].continueTarget = continueTargetPC
		}
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
				// Store to local or global variable
				if ident, ok := binExpr.Left.(ast.IdentifierExpr); ok {
					if localIdx, ok := c.localVars[ident.Name]; ok {
						c.buf.Emit(OpStore, localIdx)
					} else {
						varIdx := c.buf.AddGlobal(ident.Name)
						c.buf.Emit(OpStoreGlobal, varIdx)
					}
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

	// Patch all break and continue instructions
	if len(c.loopStack) > 0 {
		topLoop := c.loopStack[len(c.loopStack)-1]
		breakTarget := topLoop.breakTarget       // Use stored breakTarget
		continueTarget := topLoop.continueTarget // Use stored continueTarget

		// Patch break instructions
		// OpBreak VM formula: newPC = vm.pc + offset - 1, then pc++, so final_pc = vm.pc + offset
		// To land at breakTarget: offset = breakTarget - breakIdx
		for _, breakIdx := range topLoop.breakInstructions {
			offset := breakTarget - breakIdx
			c.buf.instructions[breakIdx].Args[0] = offset
		}

		// Patch continue instructions
		for _, continueIdx := range topLoop.continueInstructions {
			offset := continueTarget - continueIdx
			c.buf.instructions[continueIdx].Args[0] = offset
		}

		// Pop the loop context
		c.loopStack = c.loopStack[:len(c.loopStack)-1]
	}

	return nil
}

// compileWhileLoop compiles a while loop
func (c *Compiler) compileWhileLoop(stmt *ast.WhileStmt) error {
	// Record the start of the loop
	loopStart := len(c.buf.instructions)

	// Push loop context
	// For while loop: continue jumps to condition (loop start), break jumps past loop
	c.loopStack = append(c.loopStack, LoopContext{
		startPC:              loopStart,
		continueTarget:       loopStart, // Continue goes to condition check
		breakTarget:          0,         // Will be set after loop body
		breakInstructions:    []int{},
		continueInstructions: []int{},
	})

	// Compile condition
	if stmt.Condition != nil {
		if err := c.compileExpression(stmt.Condition); err != nil {
			return err
		}
		// Emit jump if false (placeholder offset)
		jmpIfFalseIdx := len(c.buf.instructions)
		c.buf.Emit(OpJmpIfFalse, 0)

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

		// Update loop context with correct break target
		if len(c.loopStack) > 0 {
			c.loopStack[len(c.loopStack)-1].breakTarget = len(c.buf.instructions)
		}
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

		// Set break target for infinite loop
		if len(c.loopStack) > 0 {
			c.loopStack[len(c.loopStack)-1].breakTarget = len(c.buf.instructions)
		}
	}

	// Patch all break and continue instructions
	if len(c.loopStack) > 0 {
		topLoop := c.loopStack[len(c.loopStack)-1]
		breakTarget := topLoop.breakTarget       // Jump past the loop
		continueTarget := topLoop.continueTarget // Jump back to condition

		// Patch break instructions
		// OpBreak VM formula: newPC = vm.pc + offset - 1, then pc++, so final_pc = vm.pc + offset
		// To land at breakTarget: offset = breakTarget - breakIdx
		for _, breakIdx := range topLoop.breakInstructions {
			offset := breakTarget - breakIdx
			c.buf.instructions[breakIdx].Args[0] = offset
		}

		// Patch continue instructions
		for _, continueIdx := range topLoop.continueInstructions {
			offset := continueTarget - continueIdx
			c.buf.instructions[continueIdx].Args[0] = offset
		}

		// Pop the loop context
		c.loopStack = c.loopStack[:len(c.loopStack)-1]
	}

	return nil
}

// compileForInLoop compiles a for-in loop: for x in arr { body }
func (c *Compiler) compileForInLoop(stmt *ast.ForInStmt) error {
	// Compile: let __iter = iterable; let __idx = 0; let __len = len(__iter)
	// while __idx < __len { let x = __iter[__idx]; body; __idx++ }

	// Allocate hidden loop variables
	var iterIdx, loopIdxIdx, loopLenIdx int
	if c.isInFunction {
		iterIdx = c.localVarOffset
		c.localVars["__iter_"+stmt.VarName] = iterIdx
		c.localVarOffset++

		loopIdxIdx = c.localVarOffset
		c.localVars["__forin_idx_"+stmt.VarName] = loopIdxIdx
		c.localVarOffset++

		loopLenIdx = c.localVarOffset
		c.localVars["__forin_len_"+stmt.VarName] = loopLenIdx
		c.localVarOffset++
	}

	// Store the iterable
	if err := c.compileExpression(stmt.Iterable); err != nil {
		return err
	}
	if c.isInFunction {
		c.buf.Emit(OpStore, iterIdx)
	} else {
		iterGlobal := c.buf.AddGlobal("__iter_" + stmt.VarName)
		c.buf.Emit(OpStoreGlobal, iterGlobal)
		_ = iterGlobal
	}

	// Store index = 0
	idxConst := c.buf.AddConstant(0)
	c.buf.Emit(OpPush, idxConst)
	if c.isInFunction {
		c.buf.Emit(OpStore, loopIdxIdx)
	} else {
		idxGlobal := c.buf.AddGlobal("__forin_idx_" + stmt.VarName)
		c.buf.Emit(OpStoreGlobal, idxGlobal)
		_ = idxGlobal
	}

	// Store len = len(iterable)
	if c.isInFunction {
		c.buf.Emit(OpLoad, iterIdx)
	} else {
		iterGlobal := c.buf.AddGlobal("__iter_" + stmt.VarName)
		c.buf.Emit(OpLoadGlobal, iterGlobal)
	}
	c.buf.Emit(OpArrayLen)
	if c.isInFunction {
		c.buf.Emit(OpStore, loopLenIdx)
	} else {
		lenGlobal := c.buf.AddGlobal("__forin_len_" + stmt.VarName)
		c.buf.Emit(OpStoreGlobal, lenGlobal)
		_ = lenGlobal
	}

	// Loop start: check idx < len
	loopStart := len(c.buf.instructions)

	// Push loop context
	c.loopStack = append(c.loopStack, LoopContext{
		startPC:              loopStart,
		continueTarget:       loopStart,
		breakTarget:          0,
		breakInstructions:    []int{},
		continueInstructions: []int{},
	})

	// Condition: __forin_idx < __forin_len
	if c.isInFunction {
		c.buf.Emit(OpLoad, loopIdxIdx)
		c.buf.Emit(OpLoad, loopLenIdx)
	} else {
		idxGlobal := c.buf.AddGlobal("__forin_idx_" + stmt.VarName)
		lenGlobal := c.buf.AddGlobal("__forin_len_" + stmt.VarName)
		c.buf.Emit(OpLoadGlobal, idxGlobal)
		c.buf.Emit(OpLoadGlobal, lenGlobal)
	}
	c.buf.Emit(OpLt)
	jmpExitIdx := len(c.buf.instructions)
	c.buf.Emit(OpJmpIfFalse, 0)

	// Bind loop variable: let varName = iterable[idx]
	if c.isInFunction {
		c.buf.Emit(OpLoad, iterIdx)
		c.buf.Emit(OpLoad, loopIdxIdx)
	} else {
		iterGlobal := c.buf.AddGlobal("__iter_" + stmt.VarName)
		idxGlobal := c.buf.AddGlobal("__forin_idx_" + stmt.VarName)
		c.buf.Emit(OpLoadGlobal, iterGlobal)
		c.buf.Emit(OpLoadGlobal, idxGlobal)
	}
	c.buf.Emit(OpArrayAccess)
	if c.isInFunction {
		// Allocate or reuse loop var slot
		if _, exists := c.localVars[stmt.VarName]; !exists {
			c.localVars[stmt.VarName] = c.localVarOffset
			c.localVarOffset++
		}
		c.buf.Emit(OpStore, c.localVars[stmt.VarName])
	} else {
		varGlobal := c.buf.AddGlobal(stmt.VarName)
		c.buf.Emit(OpStoreGlobal, varGlobal)
	}

	// Compile body
	for _, s := range stmt.Body {
		if err := c.compileStatement(s); err != nil {
			return err
		}
	}

	// Increment index
	if c.isInFunction {
		c.buf.Emit(OpLoad, loopIdxIdx)
	} else {
		idxGlobal := c.buf.AddGlobal("__forin_idx_" + stmt.VarName)
		c.buf.Emit(OpLoadGlobal, idxGlobal)
	}
	c.buf.Emit(OpIncr)
	if c.isInFunction {
		c.buf.Emit(OpStore, loopIdxIdx)
	} else {
		idxGlobal := c.buf.AddGlobal("__forin_idx_" + stmt.VarName)
		c.buf.Emit(OpStoreGlobal, idxGlobal)
	}

	// Jump back
	jmpOffset := loopStart - len(c.buf.instructions) - 1
	c.buf.Emit(OpJmp, jmpOffset)

	// Patch exit jump
	exitOffset := len(c.buf.instructions) - jmpExitIdx
	c.buf.instructions[jmpExitIdx].Args[0] = exitOffset

	// Patch break/continue
	if len(c.loopStack) > 0 {
		topLoop := c.loopStack[len(c.loopStack)-1]
		breakTarget := len(c.buf.instructions)
		continueTarget := topLoop.continueTarget
		for _, breakIdx := range topLoop.breakInstructions {
			c.buf.instructions[breakIdx].Args[0] = breakTarget - breakIdx
		}
		for _, continueIdx := range topLoop.continueInstructions {
			c.buf.instructions[continueIdx].Args[0] = continueTarget - continueIdx
		}
		c.loopStack = c.loopStack[:len(c.loopStack)-1]
	}

	// Clean up hidden loop variables from localVars
	if c.isInFunction {
		delete(c.localVars, "__iter_"+stmt.VarName)
		delete(c.localVars, "__forin_idx_"+stmt.VarName)
		delete(c.localVars, "__forin_len_"+stmt.VarName)
	}

	return nil
}

// compileMatchStmt compiles a match statement
func (c *Compiler) compileMatchStmt(stmt *ast.MatchStmt) error {
	if len(stmt.Cases) == 0 {
		return nil
	}

	// Compile the match expression once
	if err := c.compileExpression(stmt.Expr); err != nil {
		return err
	}

	// For each case: dup the match value, compile pattern, compare, jump if not equal
	jmpEndIdxs := []int{}

	for _, mc := range stmt.Cases {
		switch pat := mc.Pattern.(type) {
		case ast.WildcardPattern:
			// Wildcard always matches — pop match value and compile body
			c.buf.Emit(OpPop) // pop match value
			if err := c.compileMatchBody(mc.Body); err != nil {
				return err
			}
			// Jump to end (no need to check further)
			jmpEndIdx := len(c.buf.instructions)
			c.buf.Emit(OpJmp, 0)
			jmpEndIdxs = append(jmpEndIdxs, jmpEndIdx)
			// Skip remaining cases
			goto patchEnd

		case ast.LiteralPattern:
			// Dup match value, push pattern value, compare
			c.buf.Emit(OpDup)
			patConst := c.buf.AddConstant(pat.Value)
			c.buf.Emit(OpPush, patConst)
			c.buf.Emit(OpEq)
			// Jump over body if not equal
			jmpNotMatchIdx := len(c.buf.instructions)
			c.buf.Emit(OpJmpIfFalse, 0)
			// Pop the (duplicated) match value before executing body
			c.buf.Emit(OpPop)
			if err := c.compileMatchBody(mc.Body); err != nil {
				return err
			}
			// Jump to end
			jmpEndIdx := len(c.buf.instructions)
			c.buf.Emit(OpJmp, 0)
			jmpEndIdxs = append(jmpEndIdxs, jmpEndIdx)
			// Patch not-match jump
			notMatchOffset := len(c.buf.instructions) - jmpNotMatchIdx
			c.buf.instructions[jmpNotMatchIdx].Args[0] = notMatchOffset

		case ast.IdentifierPattern:
			// Bind the match value to the identifier and execute body
			c.buf.Emit(OpDup) // Keep match value for pattern binding
			// Store match value into the identifier
			if c.isInFunction {
				if _, exists := c.localVars[pat.Name]; !exists {
					c.localVars[pat.Name] = c.localVarOffset
					c.localVarOffset++
				}
				c.buf.Emit(OpStore, c.localVars[pat.Name])
			} else {
				varGlobal := c.buf.AddGlobal(pat.Name)
				c.buf.Emit(OpStoreGlobal, varGlobal)
			}
			c.buf.Emit(OpPop) // Pop remaining dup
			if err := c.compileMatchBody(mc.Body); err != nil {
				return err
			}
			jmpEndIdx := len(c.buf.instructions)
			c.buf.Emit(OpJmp, 0)
			jmpEndIdxs = append(jmpEndIdxs, jmpEndIdx)
			goto patchEnd

		default:
			// Unknown pattern - skip
			c.buf.Emit(OpPop)
		}
	}

patchEnd:
	// Pop the original match value if no case consumed it (possible if no wildcard/ident at end)
	c.buf.Emit(OpPop) // pop match value (may be no-op if already consumed)

	// Patch all end jumps
	endPC := len(c.buf.instructions)
	for _, idx := range jmpEndIdxs {
		c.buf.instructions[idx].Args[0] = endPC - idx
	}

	return nil
}

// compileMatchBody compiles the body expression of a match arm
func (c *Compiler) compileMatchBody(body ast.Expression) error {
	if err := c.compileExpression(body); err != nil {
		return err
	}
	c.buf.Emit(OpPrint)
	return nil
}

// compileTryStmt compiles a try/catch statement
func (c *Compiler) compileTryStmt(stmt *ast.TryStmt) error {
	// Emit TryBegin with placeholder offset to catch block
	tryBeginIdx := len(c.buf.instructions)
	c.buf.Emit(OpTryBegin, 0)

	// Compile try body
	for _, inner := range stmt.Body {
		if err := c.compileStatement(inner); err != nil {
			return err
		}
	}

	// Emit TryEnd and jump over catch block
	c.buf.Emit(OpTryEnd)
	jmpOverCatchIdx := len(c.buf.instructions)
	c.buf.Emit(OpJmp, 0)

	// Patch TryBegin to point to catch block
	catchPC := len(c.buf.instructions)
	catchOffset := catchPC - tryBeginIdx
	c.buf.instructions[tryBeginIdx].Args[0] = catchOffset

	// Compile catch body
	for _, inner := range stmt.Catch {
		if err := c.compileStatement(inner); err != nil {
			return err
		}
	}

	// Patch jump over catch
	endOffset := len(c.buf.instructions) - jmpOverCatchIdx
	c.buf.instructions[jmpOverCatchIdx].Args[0] = endOffset

	return nil
}

// compileDeferStmt compiles a defer statement
func (c *Compiler) compileDeferStmt(stmt *ast.DeferStmt) error {
	// Emit OpDefer with a jump-over offset so the deferred block is skipped
	// on normal execution, but will be called on return.
	// We encode deferred calls as inline deferred sections.
	deferIdx := len(c.buf.instructions)
	c.buf.Emit(OpDefer, 0)

	// Save start of deferred code
	deferCodeStart := len(c.buf.instructions)

	// Compile the deferred call
	if err := c.compileExpression(stmt.Call); err != nil {
		return err
	}
	// Pop the return value of the deferred call
	c.buf.Emit(OpPop)
	// Emit a return from deferred section (OpNoop — VM pops deferred context)
	c.buf.Emit(OpNoop)

	// Patch OpDefer to skip over the deferred block
	deferEnd := len(c.buf.instructions)
	c.buf.instructions[deferIdx].Args[0] = deferEnd - deferCodeStart

	return nil
}

// compileFunctionBody compiles the body of a function declaration
func (c *Compiler) compileFunctionBody(fnStmt *ast.FunctionDefStmt) error {
	// Register function entry point
	_ = c.buf.RegisterFunction(fnStmt.Name)

	// Save current state
	oldLocalVars := c.localVars
	oldFuncAST := c.currentFuncAST
	oldIsInFunc := c.isInFunction
	oldVarOffset := c.localVarOffset

	// Set up new scope for function
	c.localVars = make(map[string]int)
	c.currentFuncAST = fnStmt
	c.isInFunction = true
	c.localVarOffset = 0

	// Map parameters to local variable indices (0-indexed)
	for i, param := range fnStmt.Params {
		c.localVars[param.Name] = i
	}
	// Local variables start AFTER parameter slots to avoid overwriting args
	c.localVarOffset = len(fnStmt.Params)

	if fnStmt.IsExprBody {
		// Expression-bodied function: compile expression
		if expr, ok := fnStmt.Body.(ast.Expression); ok {
			if err := c.compileExpression(expr); err != nil {
				return err
			}
		}
	} else {
		// Block-bodied function: compile all statements uniformly.
		// compileStatement handles ReturnStmt via compileReturnStatement which emits OpReturnValue.
		// An implicit nil return is appended at the end; it is only reached at runtime
		// if no explicit return statement was executed.
		if stmts, ok := fnStmt.Body.([]ast.Statement); ok {
			for _, stmt := range stmts {
				if err := c.compileStatement(stmt); err != nil {
					return err
				}
			}
			// Implicit nil return — only reached if no explicit return executed
			nilIdx := c.buf.AddConstant(nil)
			c.buf.Emit(OpPush, nilIdx)
			c.buf.Emit(OpReturnValue)
		}
	}

	// Emit return instruction (for expression-bodied functions)
	if fnStmt.IsExprBody {
		c.buf.Emit(OpReturnValue)
	}

	// Restore state
	c.localVars = oldLocalVars
	c.currentFuncAST = oldFuncAST
	c.isInFunction = oldIsInFunc
	c.localVarOffset = oldVarOffset

	return nil
}

// compileFunctionDeclaration is called from compileStatement for function declarations
// Functions are already compiled during the Compile phase, so this is a no-op
func (c *Compiler) compileFunctionDeclaration(fn *ast.FunctionDefStmt) error {
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
		// Check if it's a local variable (function parameter) first
		if localIdx, ok := c.localVars[e.Name]; ok {
			c.buf.Emit(OpLoad, localIdx)
		} else {
			varIdx := c.buf.AddGlobal(e.Name)
			c.buf.Emit(OpLoadGlobal, varIdx)
		}
	case ast.NilExpr:
		idx := c.buf.AddConstant(nil)
		c.buf.Emit(OpPush, idx)
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
	case ast.ClosureExpr:
		// Compile the closure body as an anonymous function that captures outer-scope
		// variables as hidden trailing parameters (lexical closure via value capture).
		name := fmt.Sprintf("__closure_%d", c.closureCounter)
		c.closureCounter++

		// Find free variables (referenced in body, not in params, but in outer scope).
		paramSet := make(map[string]bool)
		for _, p := range e.Params {
			paramSet[p.Name] = true
		}
		freeVars := c.collectFreeVars(e.Body, paramSet, c.localVars)

		// Build extended parameter list: original params + hidden capture params.
		extParams := make([]ast.Param, len(e.Params))
		copy(extParams, e.Params)
		for _, fv := range freeVars {
			extParams = append(extParams, ast.Param{Name: fv})
		}

		fnStmt := &ast.FunctionDefStmt{
			Name:   name,
			Params: extParams,
			Body:   e.Body,
		}
		c.astFunctions[name] = fnStmt

		// Emit jump-over so main code skips the function body
		jmpIdx := len(c.buf.instructions)
		c.buf.Emit(OpJmp, 0) // patched after body

		// Compile the function body inline
		if err := c.compileFunctionBody(fnStmt); err != nil {
			return err
		}

		// Patch the jump to land right after the function body
		afterBodyPC := len(c.buf.instructions)
		c.buf.instructions[jmpIdx].Args[0] = afterBodyPC - jmpIdx

		if len(freeVars) == 0 {
			// No captures — push function name string as handle (cheaper path)
			idx := c.buf.AddConstant(name)
			c.buf.Emit(OpPush, idx)
		} else {
			// Push captured var values from outer scope, then create ClosureValue
			for _, varName := range freeVars {
				if localIdx, ok := c.localVars[varName]; ok {
					c.buf.Emit(OpLoad, localIdx)
				} else {
					gIdx := c.buf.AddGlobal(varName)
					c.buf.Emit(OpLoadGlobal, gIdx)
				}
			}
			c.buf.Emit(OpMakeClosure, name, len(freeVars))
		}
		return nil
	case ast.ChanExpr:
		// Channels not yet implemented — push nil placeholder
		idx := c.buf.AddConstant(nil)
		c.buf.Emit(OpPush, idx)
		return nil
	case ast.StringInterpExpr:
		return c.compileStringInterp(&e)
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

	case ast.OpPreInc:
		// Handle prefix increment: ++i
		// Increments variable and returns new value
		if ident, ok := expr.Expr.(ast.IdentifierExpr); ok {
			if localIdx, ok2 := c.localVars[ident.Name]; ok2 {
				c.buf.Emit(OpLoad, localIdx)
				c.buf.Emit(OpIncr)
				c.buf.Emit(OpStore, localIdx)
				c.buf.Emit(OpLoad, localIdx)
			} else {
				varIdx := c.buf.AddGlobal(ident.Name)
				c.buf.Emit(OpLoadGlobal, varIdx)
				c.buf.Emit(OpIncr)
				c.buf.Emit(OpStoreGlobal, varIdx)
				c.buf.Emit(OpLoadGlobal, varIdx)
			}
		} else {
			if err := c.compileExpression(expr.Expr); err != nil {
				return err
			}
			c.buf.Emit(OpIncr)
		}

	case ast.OpPreDec:
		// Handle prefix decrement: --i
		// Decrements variable and returns new value
		if ident, ok := expr.Expr.(ast.IdentifierExpr); ok {
			if localIdx, ok2 := c.localVars[ident.Name]; ok2 {
				c.buf.Emit(OpLoad, localIdx)
				c.buf.Emit(OpDecr)
				c.buf.Emit(OpStore, localIdx)
				c.buf.Emit(OpLoad, localIdx)
			} else {
				varIdx := c.buf.AddGlobal(ident.Name)
				c.buf.Emit(OpLoadGlobal, varIdx)
				c.buf.Emit(OpDecr)
				c.buf.Emit(OpStoreGlobal, varIdx)
				c.buf.Emit(OpLoadGlobal, varIdx)
			}
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

		// Handle array assignment: arr[i] = val is lowered to __array_assign(arr, i, val)
		if ident.Name == "__array_assign" {
			for _, arg := range expr.Arguments {
				if err := c.compileExpression(arg); err != nil {
					return err
				}
			}
			// OpArrayStore: pops (value, index, array) → pushes modified array
			c.buf.Emit(OpArrayStore)
			return nil
		}

		// Check if it's a user-defined function
		if _, ok := c.astFunctions[ident.Name]; ok {
			// Emit OpCall for user-defined functions
			// Arguments are compiled and pushed on the stack
			for _, arg := range expr.Arguments {
				if err := c.compileExpression(arg); err != nil {
					return err
				}
			}
			// Emit OpCall with function name and argument count
			// The return value will be on the stack after the call
			c.buf.Emit(OpCall, ident.Name, len(expr.Arguments))
			return nil
		}

		// Compile arguments for unknown functions (might be built-in or closure variable)
		// If the identifier resolves to a LOCAL variable it holds a closure handle — use OpCallDynamic.
		if localIdx, isLocal := c.localVars[ident.Name]; isLocal {
			// Push the closure handle (stored in local var), then push args, then call dynamically.
			c.buf.Emit(OpLoad, localIdx)
			for _, arg := range expr.Arguments {
				if err := c.compileExpression(arg); err != nil {
					return err
				}
			}
			c.buf.Emit(OpCallDynamic, len(expr.Arguments))
			return nil
		}

		// Otherwise: global closure var or built-in — use OpCall; VM resolves at runtime.
		for _, arg := range expr.Arguments {
			if err := c.compileExpression(arg); err != nil {
				return err
			}
		}

		c.buf.Emit(OpCall, ident.Name, len(expr.Arguments))
	} else if mem, ok := expr.Callee.(ast.MemberAccessExpr); ok {
		// Method call on an object: obj.method(args...)
		switch mem.Member {
		case "push":
			// arr.push(val) — append val to arr and store back
			if objIdent, ok2 := mem.Object.(ast.IdentifierExpr); ok2 {
				// Compile the value to push
				if len(expr.Arguments) > 0 {
					if err := c.compileExpression(expr.Arguments[0]); err != nil {
						return err
					}
				} else {
					nilIdx := c.buf.AddConstant(nil)
					c.buf.Emit(OpPush, nilIdx)
				}
				// Load the array
				if localIdx, ok3 := c.localVars[objIdent.Name]; ok3 {
					c.buf.Emit(OpLoad, localIdx)
				} else {
					arrIdx := c.buf.AddGlobal(objIdent.Name)
					c.buf.Emit(OpLoadGlobal, arrIdx)
				}
				// OpArrayPush: pops (array, value) and pushes new_array
				c.buf.Emit(OpArrayPush)
				// Store new array back
				if localIdx, ok3 := c.localVars[objIdent.Name]; ok3 {
					c.buf.Emit(OpStore, localIdx)
				} else {
					arrIdx := c.buf.AddGlobal(objIdent.Name)
					c.buf.Emit(OpStoreGlobal, arrIdx)
				}
				// Push nil as call return value
				nilIdx := c.buf.AddConstant(nil)
				c.buf.Emit(OpPush, nilIdx)
			} else {
				// Non-identifier — compile object and args, emit generic call
				if err := c.compileExpression(mem.Object); err != nil {
					return err
				}
				for _, arg := range expr.Arguments {
					if err := c.compileExpression(arg); err != nil {
						return err
					}
				}
				c.buf.Emit(OpArrayPush)
			}
		case "pop":
			// arr.pop() — remove last element, store new array back, return popped value
			if objIdent, ok2 := mem.Object.(ast.IdentifierExpr); ok2 {
				// Load the array
				if localIdx, ok3 := c.localVars[objIdent.Name]; ok3 {
					c.buf.Emit(OpLoad, localIdx)
				} else {
					arrIdx := c.buf.AddGlobal(objIdent.Name)
					c.buf.Emit(OpLoadGlobal, arrIdx)
				}
				// OpArrayPop: pops array, pushes (new_array, popped_value)
				c.buf.Emit(OpArrayPop)
				// Stack is now: [..., popped_value, new_array]
				// Store new array back (it's on top)
				if localIdx, ok3 := c.localVars[objIdent.Name]; ok3 {
					c.buf.Emit(OpStore, localIdx)
				} else {
					arrIdx := c.buf.AddGlobal(objIdent.Name)
					c.buf.Emit(OpStoreGlobal, arrIdx)
				}
				// popped_value is now on top (the "return value")
			} else {
				if err := c.compileExpression(mem.Object); err != nil {
					return err
				}
				c.buf.Emit(OpArrayPop)
			}
		default:
			// Check if this is a module-qualified call like math.sqrt(x)
			if objIdent, ok2 := mem.Object.(ast.IdentifierExpr); ok2 {
				switch objIdent.Name {
				case "math":
					// math.xxx(args) → OpCall("math.xxx", argCount)
					for _, arg := range expr.Arguments {
						if err := c.compileExpression(arg); err != nil {
							return err
						}
					}
					c.buf.Emit(OpCall, "math."+mem.Member, len(expr.Arguments))
					return nil
				}
			}
			// General method call: push receiver, push args, OpMethodCall
			if err := c.compileExpression(mem.Object); err != nil {
				return err
			}
			for _, arg := range expr.Arguments {
				if err := c.compileExpression(arg); err != nil {
					return err
				}
			}
			c.buf.Emit(OpMethodCall, mem.Member, len(expr.Arguments))
		}
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
	// Special case: math module constants (math.pi, math.e)
	if objIdent, ok := expr.Object.(ast.IdentifierExpr); ok && objIdent.Name == "math" {
		switch expr.Member {
		case "pi":
			idx := c.buf.AddConstant(3.141592653589793)
			c.buf.Emit(OpPush, idx)
			return nil
		case "e":
			idx := c.buf.AddConstant(2.718281828459045)
			c.buf.Emit(OpPush, idx)
			return nil
		}
	}
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
	// Sort field names for deterministic ordering (map iteration is random in Go)
	fieldNames := make([]string, 0, len(expr.Fields))
	for name := range expr.Fields {
		fieldNames = append(fieldNames, name)
	}
	sort.Strings(fieldNames)

	// For each field in deterministic order, compile the value and push to stack
	for _, fieldName := range fieldNames {
		fieldVal := expr.Fields[fieldName]
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
	hasValue := expr.Value != nil
	// Compile the enum variant value if provided
	if hasValue {
		if err := c.compileExpression(expr.Value); err != nil {
			return err
		}
	}

	// Emit enum variant instruction with hasValue flag so VM knows to pop the value
	enumIdx := c.buf.AddConstant(expr.EnumName)
	variantIdx := c.buf.AddConstant(expr.Variant)
	c.buf.Emit(OpEnumVariant, enumIdx, variantIdx, hasValue)
	return nil
}

// compileStringInterp compiles an interpolated string by concatenating all parts using OpAdd.
// e.g., "Hello, ${name}!" → push "Hello, " / load name / OpAdd / push "!" / OpAdd
func (c *Compiler) compileStringInterp(expr *ast.StringInterpExpr) error {
	if len(expr.Parts) == 0 {
		idx := c.buf.AddConstant("")
		c.buf.Emit(OpPush, idx)
		return nil
	}

	// Push the first part to start the chain
	first := expr.Parts[0]
	if !first.IsExpr {
		idx := c.buf.AddConstant(first.Literal)
		c.buf.Emit(OpPush, idx)
	} else {
		// Start with an empty string accumulator so that OpAdd produces a string
		emptyIdx := c.buf.AddConstant("")
		c.buf.Emit(OpPush, emptyIdx)
		if err := c.compileExpression(first.Expr); err != nil {
			return err
		}
		c.buf.Emit(OpAdd)
	}

	// Concatenate remaining parts
	for _, part := range expr.Parts[1:] {
		if !part.IsExpr {
			idx := c.buf.AddConstant(part.Literal)
			c.buf.Emit(OpPush, idx)
		} else {
			if err := c.compileExpression(part.Expr); err != nil {
				return err
			}
		}
		c.buf.Emit(OpAdd)
	}
	return nil
}

// collectFreeVars walks the closure body ([]ast.Statement or ast.Expression),
// collects all variable names that are referenced but not declared locally
// and appear in outerLocals. The paramSet contains the closure's own parameter names.
// Returns an ordered, deduplicated slice of captured variable names.
func (c *Compiler) collectFreeVars(body interface{}, paramSet map[string]bool, outerLocals map[string]int) []string {
	refs := make(map[string]bool)
	decls := make(map[string]bool)
	for k := range paramSet {
		decls[k] = true
	}

	switch b := body.(type) {
	case []ast.Statement:
		for _, s := range b {
			c.freeVarWalkStmt(s, refs, decls)
		}
	case ast.Expression:
		c.freeVarWalkExpr(b, refs, decls)
	}

	seen := make(map[string]bool)
	var result []string
	for name := range refs {
		if _, inOuter := outerLocals[name]; inOuter {
			if !decls[name] && !seen[name] {
				seen[name] = true
				result = append(result, name)
			}
		}
	}
	return result
}

func (c *Compiler) freeVarWalkStmt(stmt ast.Statement, refs, decls map[string]bool) {
	if stmt == nil {
		return
	}
	switch s := stmt.(type) {
	case ast.LetStmt:
		if s.Value != nil {
			c.freeVarWalkExpr(s.Value, refs, decls)
		}
		decls[s.Name] = true
	case ast.AssignStmt:
		c.freeVarWalkExpr(s.Value, refs, decls)
		decls[s.Name] = true
	case ast.CompoundAssignStmt:
		c.freeVarWalkExpr(s.Value, refs, decls)
		refs[s.Name] = true
	case ast.ExprStmt:
		c.freeVarWalkExpr(s.Expr, refs, decls)
	case ast.ReturnStmt:
		if s.Value != nil {
			c.freeVarWalkExpr(s.Value, refs, decls)
		}
	case ast.IfStmt:
		c.freeVarWalkExpr(s.Condition, refs, decls)
		for _, st := range s.ThenBranch {
			c.freeVarWalkStmt(st, refs, decls)
		}
		for _, st := range s.ElseBranch {
			c.freeVarWalkStmt(st, refs, decls)
		}
	case ast.WhileStmt:
		c.freeVarWalkExpr(s.Condition, refs, decls)
		for _, st := range s.Body {
			c.freeVarWalkStmt(st, refs, decls)
		}
	case ast.ForStmt:
		// ForStmt has Init/Condition/Incr/Body — no iterable
		for _, st := range s.Body {
			c.freeVarWalkStmt(st, refs, decls)
		}
	case ast.ForInStmt:
		c.freeVarWalkExpr(s.Iterable, refs, decls)
		decls[s.VarName] = true
		for _, st := range s.Body {
			c.freeVarWalkStmt(st, refs, decls)
		}
	case ast.BlockStmt:
		for _, st := range s.Statements {
			c.freeVarWalkStmt(st, refs, decls)
		}
	case ast.PrintStmt:
		c.freeVarWalkExpr(s.Value, refs, decls)
	case ast.FunctionDefStmt:
		decls[s.Name] = true
	}
}

func (c *Compiler) freeVarWalkExpr(expr ast.Expression, refs, decls map[string]bool) {
	if expr == nil {
		return
	}
	switch e := expr.(type) {
	case ast.IdentifierExpr:
		refs[e.Name] = true
	case ast.BinaryExpr:
		c.freeVarWalkExpr(e.Left, refs, decls)
		c.freeVarWalkExpr(e.Right, refs, decls)
	case ast.UnaryExpr:
		c.freeVarWalkExpr(e.Expr, refs, decls)
	case ast.CallExpr:
		c.freeVarWalkExpr(e.Callee, refs, decls)
		for _, arg := range e.Arguments {
			c.freeVarWalkExpr(arg, refs, decls)
		}
	case ast.MemberAccessExpr:
		c.freeVarWalkExpr(e.Object, refs, decls)
	case ast.ArrayAccessExpr:
		c.freeVarWalkExpr(e.Array, refs, decls)
		c.freeVarWalkExpr(e.Index, refs, decls)
	case ast.ArrayExpr:
		for _, el := range e.Elements {
			c.freeVarWalkExpr(el, refs, decls)
		}
	case ast.ClosureExpr:
		innerDecls := make(map[string]bool)
		for k := range decls {
			innerDecls[k] = true
		}
		for _, p := range e.Params {
			innerDecls[p.Name] = true
		}
		for _, s := range e.Body {
			c.freeVarWalkStmt(s, refs, innerDecls)
		}
	case ast.StringInterpExpr:
		for _, part := range e.Parts {
			if part.IsExpr {
				c.freeVarWalkExpr(part.Expr, refs, decls)
			}
		}
	}
}
