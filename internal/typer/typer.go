package typer

import (
	"fmt"

	"github.com/ecocee/kode-go/pkg/ast"
)

// TypeVar represents a type variable for inference
type TypeVar struct {
	ID int
}

func (tv TypeVar) String() string { return fmt.Sprintf("T%d", tv.ID) }

// Constraint represents a type constraint
type Constraint struct {
	Left  ast.Type
	Right ast.Type
}

// Substitution map
type Substitution map[TypeVar]ast.Type

// Typer performs type checking and inference
type Typer struct {
	nextVarID   int
	env         map[string]ast.Type
	constraints []Constraint
}

// NewTyper creates a new type checker
func NewTyper() *Typer {
	typer := &Typer{
		nextVarID:   0,
		env:         make(map[string]ast.Type),
		constraints: []Constraint{},
	}

	// Add built-in functions
	typer.env["print"] = ast.FunctionType{
		ParamTypes: []ast.Type{ast.StringType{}},
		ReturnType: ast.VoidType{},
	}

	return typer
}

// CheckProgram type checks a program
func (t *Typer) CheckProgram(program ast.Program) error {
	for _, stmt := range program.Statements {
		if err := t.checkStatement(stmt); err != nil {
			return err
		}
	}
	return t.solveConstraints()
}

// checkStatement type checks a statement
func (t *Typer) checkStatement(stmt ast.Statement) error {
	switch s := stmt.(type) {
	case ast.LetStmt:
		typ, err := t.inferExpression(s.Value)
		if err != nil {
			return err
		}
		if s.Type != nil {
			t.addConstraint(s.Type, typ)
		} else {
			// Infer type
			s.Type = typ
		}
		t.env[s.Name] = s.Type
	case ast.FunctionDefStmt:
		// Add function to env
		paramTypes := make([]ast.Type, len(s.Params))
		for i, param := range s.Params {
			paramTypes[i] = param.Type
		}
		fnType := ast.FunctionType{ParamTypes: paramTypes, ReturnType: s.ReturnType}
		t.env[s.Name] = fnType
		// Check body in new scope
		oldEnv := t.env
		t.env = make(map[string]ast.Type)
		for k, v := range oldEnv {
			t.env[k] = v
		}
		for _, param := range s.Params {
			t.env[param.Name] = param.Type
		}
		if s.IsExprBody {
			// Body is an Expression
			if _, err := t.inferExpression(s.Body.(ast.Expression)); err != nil {
				return err
			}
		} else {
			// Body is []Statement
			for _, bodyStmt := range s.Body.([]ast.Statement) {
				if err := t.checkStatement(bodyStmt); err != nil {
					return err
				}
			}
		}
		t.env = oldEnv
	case ast.ReturnStmt:
		// Assume return type from context
	case ast.IfStmt:
		condType, err := t.inferExpression(s.Condition)
		if err != nil {
			return err
		}
		t.addConstraint(ast.BoolType{}, condType)
		for _, stmt := range s.ThenBranch {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
		for _, stmt := range s.ElseBranch {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
	case ast.WhileStmt:
		condType, err := t.inferExpression(s.Condition)
		if err != nil {
			return err
		}
		t.addConstraint(ast.BoolType{}, condType)
		for _, stmt := range s.Body {
			if err := t.checkStatement(stmt); err != nil {
				return err
			}
		}
	case ast.PrintStmt:
		_, err := t.inferExpression(s.Value)
		return err
	case ast.ExprStmt:
		_, err := t.inferExpression(s.Expr)
		return err
		// Add more cases as needed
	}
	return nil
}

// inferExpression infers the type of an expression
func (t *Typer) inferExpression(expr ast.Expression) (ast.Type, error) {
	switch e := expr.(type) {
	case ast.NumberExpr:
		return ast.IntType{}, nil
	case ast.FloatExpr:
		return ast.FloatType{}, nil
	case ast.BoolExpr:
		return ast.BoolType{}, nil
	case ast.StringExpr:
		return ast.StringType{}, nil
	case ast.IdentifierExpr:
		if typ, ok := t.env[e.Name]; ok {
			return typ, nil
		}
		return nil, fmt.Errorf("undefined variable: %s", e.Name)
	case ast.BinaryExpr:
		leftType, err := t.inferExpression(e.Left)
		if err != nil {
			return nil, err
		}
		rightType, err := t.inferExpression(e.Right)
		if err != nil {
			return nil, err
		}
		switch e.Op {
		case ast.OpAdd, ast.OpSubtract, ast.OpMultiply, ast.OpDivide:
			// Allow both int and float for arithmetic
			// Determine result type based on operands
			leftIsNumeric := t.isNumericType(leftType)
			rightIsNumeric := t.isNumericType(rightType)

			if !leftIsNumeric || !rightIsNumeric {
				return nil, fmt.Errorf("non-numeric operands for arithmetic")
			}

			// If either is float, result is float
			if _, isFloat := leftType.(ast.FloatType); isFloat {
				return ast.FloatType{}, nil
			}
			if _, isFloat := rightType.(ast.FloatType); isFloat {
				return ast.FloatType{}, nil
			}
			return ast.IntType{}, nil
		case ast.OpEqual, ast.OpNotEqual, ast.OpLessThan, ast.OpGreaterThan, ast.OpLessThanOrEqual, ast.OpGreaterThanOrEqual:
			t.addConstraint(leftType, rightType)
			return ast.BoolType{}, nil
		case ast.OpAnd, ast.OpOr:
			t.addConstraint(leftType, ast.BoolType{})
			t.addConstraint(rightType, ast.BoolType{})
			return ast.BoolType{}, nil
		}
	case ast.CallExpr:
		calleeType, err := t.inferExpression(e.Callee)
		if err != nil {
			return nil, err
		}

		// Special handling for built-in functions
		if idExpr, ok := e.Callee.(ast.IdentifierExpr); ok {
			if idExpr.Name == "print" {
				// print accepts any type
				for _, arg := range e.Arguments {
					_, err := t.inferExpression(arg)
					if err != nil {
						return nil, err
					}
				}
				return ast.VoidType{}, nil
			}
		}

		if fnType, ok := calleeType.(ast.FunctionType); ok {
			if len(fnType.ParamTypes) != len(e.Arguments) {
				return nil, fmt.Errorf("wrong number of arguments")
			}
			for i, arg := range e.Arguments {
				argType, err := t.inferExpression(arg)
				if err != nil {
					return nil, err
				}
				t.addConstraint(fnType.ParamTypes[i], argType)
			}
			return fnType.ReturnType, nil
		}
		return nil, fmt.Errorf("not a function")
	case ast.ArrayExpr:
		if len(e.Elements) == 0 {
			return ast.ArrayType{ElementType: t.newTypeVar()}, nil
		}
		elemType, err := t.inferExpression(e.Elements[0])
		if err != nil {
			return nil, err
		}
		for _, elem := range e.Elements[1:] {
			et, err := t.inferExpression(elem)
			if err != nil {
				return nil, err
			}
			t.addConstraint(elemType, et)
		}
		return ast.ArrayType{ElementType: elemType}, nil
		// Add more cases
	}
	return t.newTypeVar(), nil // Default to type var
}

// addConstraint adds a type constraint
func (t *Typer) addConstraint(left, right ast.Type) {
	t.constraints = append(t.constraints, Constraint{Left: left, Right: right})
}

// newTypeVar creates a new type variable
func (t *Typer) newTypeVar() TypeVar {
	tv := TypeVar{ID: t.nextVarID}
	t.nextVarID++
	return tv
}

// solveConstraints solves the constraints (simplified unification)
func (t *Typer) solveConstraints() error {
	// Simplified: just check basic constraints
	for _, c := range t.constraints {
		if !t.unify(c.Left, c.Right) {
			return fmt.Errorf("type mismatch: %v vs %v", c.Left, c.Right)
		}
	}
	return nil
}

// unify checks if two types unify (simplified)
func (t *Typer) unify(a, b ast.Type) bool {
	switch at := a.(type) {
	case ast.IntType:
		_, ok := b.(ast.IntType)
		return ok
	case ast.BoolType:
		_, ok := b.(ast.BoolType)
		return ok
	case ast.StringType:
		_, ok := b.(ast.StringType)
		return ok
	case ast.FloatType:
		_, ok := b.(ast.FloatType)
		return ok
	case ast.ArrayType:
		if bt, ok := b.(ast.ArrayType); ok {
			return t.unify(at.ElementType, bt.ElementType)
		}
		return false
	case ast.FunctionType:
		if bt, ok := b.(ast.FunctionType); ok {
			if len(at.ParamTypes) != len(bt.ParamTypes) {
				return false
			}
			for i := range at.ParamTypes {
				if !t.unify(at.ParamTypes[i], bt.ParamTypes[i]) {
					return false
				}
			}
			return t.unify(at.ReturnType, bt.ReturnType)
		}
		return false
	case TypeVar:
		// For simplicity, assume type vars unify with anything
		return true
	}
	return false
}

// isNumericType checks if a type is numeric (int or float)
func (t *Typer) isNumericType(typ ast.Type) bool {
	switch typ.(type) {
	case ast.IntType, ast.FloatType:
		return true
	default:
		return false
	}
}
