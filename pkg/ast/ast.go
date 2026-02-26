package ast

import (
	"encoding/json"
	"fmt"
	"strings"
)

// Type represents a type in the language
type Type interface {
	String() string
}

// Basic types
type IntType struct{}

func (t IntType) String() string { return "int" }

type FloatType struct{}

func (t FloatType) String() string { return "float" }

type BoolType struct{}

func (t BoolType) String() string { return "bool" }

type StringType struct{}

func (t StringType) String() string { return "string" }

type ArrayType struct {
	ElementType Type
}

func (t ArrayType) String() string { return "[]" + t.ElementType.String() }

type VoidType struct{}

func (t VoidType) String() string { return "void" }

// Additional types
type StructType struct {
	Name   string
	Fields map[string]Type
}

func (t StructType) String() string { return t.Name }

type EnumType struct {
	Name     string
	Variants []string
}

func (t EnumType) String() string { return t.Name }

type TraitType struct {
	Name string
}

func (t TraitType) String() string { return t.Name }

type FunctionType struct {
	ParamTypes []Type
	ReturnType Type
}

func (t FunctionType) String() string {
	params := make([]string, len(t.ParamTypes))
	for i, p := range t.ParamTypes {
		params[i] = p.String()
	}
	return fmt.Sprintf("fn(%s) -> %s", strings.Join(params, ", "), t.ReturnType.String())
}

// Pattern interface
type Pattern interface {
	pattern()
}

// Concrete patterns
type WildcardPattern struct{}

func (p WildcardPattern) pattern() {}

type IdentifierPattern struct {
	Name string
}

func (p IdentifierPattern) pattern() {}

type LiteralPattern struct {
	Value interface{}
}

func (p LiteralPattern) pattern() {}

// Program
type Program struct {
	Statements []Statement `json:"statements"`
}

// Statement interface
type Statement interface {
	statement()
}

// Concrete statements
type LetStmt struct {
	Name  string     `json:"name"`
	Type  Type       `json:"type,omitempty"`
	Value Expression `json:"value"`
}

func (s LetStmt) statement() {}

type AssignStmt struct {
	Name  string     `json:"name"`
	Value Expression `json:"value"`
}

func (s AssignStmt) statement() {}

type FunctionDefStmt struct {
	FilePrefix string      `json:"file_prefix"`
	IsMain     bool        `json:"is_main"`
	Name       string      `json:"name"`
	Params     []Param     `json:"params"`
	ReturnType Type        `json:"return_type,omitempty"`
	Body       interface{} `json:"body"` // []Statement or Expression
	IsExprBody bool        `json:"is_expr_body"`
}

func (s FunctionDefStmt) statement() {}

type Param struct {
	Name string `json:"name"`
	Type Type   `json:"type"`
}

type ReturnStmt struct {
	Value Expression `json:"value"`
}

func (s ReturnStmt) statement() {}

type IfStmt struct {
	Condition  Expression  `json:"condition"`
	ThenBranch []Statement `json:"then_branch"`
	ElseBranch []Statement `json:"else_branch,omitempty"`
}

func (s IfStmt) statement() {}

type WhileStmt struct {
	Condition Expression  `json:"condition"`
	Body      []Statement `json:"body"`
}

func (s WhileStmt) statement() {}

type ForStmt struct {
	Init      interface{} `json:"init,omitempty"`
	Condition Expression  `json:"condition,omitempty"`
	Incr      Expression  `json:"incr,omitempty"`
	Body      []Statement `json:"body"`
}

func (s ForStmt) statement() {}

type ExprStmt struct {
	Expr Expression `json:"expr"`
}

func (s ExprStmt) statement() {}

type BlockStmt struct {
	Statements []Statement `json:"statements"`
}

func (s BlockStmt) statement() {}

type PrintStmt struct {
	Value Expression `json:"value"`
}

func (s PrintStmt) statement() {}

type ImportStmt struct {
	Path string `json:"path"`
}

func (s ImportStmt) statement() {}

type TryStmt struct {
	Body  []Statement `json:"body"`
	Catch []Statement `json:"catch"`
}

func (s TryStmt) statement() {}

// Concurrency statements
type GoStmt struct {
	Call Expression `json:"call"`
}

func (s GoStmt) statement() {}

type SelectStmt struct {
	Cases []SelectCase `json:"cases"`
}

func (s SelectStmt) statement() {}

type SelectCase struct {
	Chan      Expression  `json:"chan,omitempty"`
	Value     string      `json:"value,omitempty"`
	Body      []Statement `json:"body"`
	IsDefault bool        `json:"is_default"`
}

// HTTP statements
type HttpStmt struct {
	Method  string     `json:"method"`
	Path    string     `json:"path"`
	Handler Expression `json:"handler"`
}

func (s HttpStmt) statement() {}

type RouteStmt struct {
	Method string      `json:"method"`
	Path   string      `json:"path"`
	Body   []Statement `json:"body"`
}

func (s RouteStmt) statement() {}

// New statements
type BreakStmt struct{}

func (s BreakStmt) statement() {}

type ContinueStmt struct{}

func (s ContinueStmt) statement() {}

type SpawnStmt struct {
	Call Expression
}

func (s SpawnStmt) statement() {}

type DeferStmt struct {
	Call Expression
}

func (s DeferStmt) statement() {}

type MatchStmt struct {
	Expr  Expression
	Cases []MatchCase
}

func (s MatchStmt) statement() {}

type MatchCase struct {
	Pattern Pattern
	Body    Expression
}

type ConstDeclStmt struct {
	Name  string
	Type  Type
	Value Expression
}

func (s ConstDeclStmt) statement() {}

type StructDeclStmt struct {
	Name   string
	Fields []Field
}

func (s StructDeclStmt) statement() {}

type Field struct {
	Name string
	Type Type
}

type EnumDeclStmt struct {
	Name     string
	Variants []string
}

func (s EnumDeclStmt) statement() {}

type TraitDeclStmt struct {
	Name       string
	Signatures []FunctionSignature
}

func (s TraitDeclStmt) statement() {}

type FunctionSignature struct {
	Name       string
	Params     []Param
	ReturnType Type
}

type ImplDeclStmt struct {
	Trait   string
	Type    string
	Methods []FunctionDefStmt
}

func (s ImplDeclStmt) statement() {}

type ServiceDeclStmt struct {
	Name   string
	Routes []RouteStmt
}

func (s ServiceDeclStmt) statement() {}

type ModuleDeclStmt struct {
	Name string
}

func (s ModuleDeclStmt) statement() {}

type Expression interface {
	expression()
}

// Concrete expressions
type NumberExpr struct {
	Value int64 `json:"value"`
}

func (e NumberExpr) expression() {}

type FloatExpr struct {
	Value float64 `json:"value"`
}

func (e FloatExpr) expression() {}

type BoolExpr struct {
	Value bool `json:"value"`
}

func (e BoolExpr) expression() {}

type StringExpr struct {
	Value string `json:"value"`
}

func (e StringExpr) expression() {}

type IdentifierExpr struct {
	Name string `json:"name"`
}

func (e IdentifierExpr) expression() {}

type BinaryExpr struct {
	Left  Expression `json:"left"`
	Op    BinaryOp   `json:"op"`
	Right Expression `json:"right"`
}

func (e BinaryExpr) expression() {}

type UnaryExpr struct {
	Op   UnaryOp    `json:"op"`
	Expr Expression `json:"expr"`
}

func (e UnaryExpr) expression() {}

type CallExpr struct {
	Callee    Expression   `json:"callee"`
	Arguments []Expression `json:"arguments"`
}

func (e CallExpr) expression() {}

type ArrayExpr struct {
	Elements []Expression `json:"elements"`
}

func (e ArrayExpr) expression() {}

type ArrayAccessExpr struct {
	Array Expression `json:"array"`
	Index Expression `json:"index"`
}

func (e ArrayAccessExpr) expression() {}

type ClosureExpr struct {
	Params []Param     `json:"params"`
	Body   []Statement `json:"body"`
}

func (e ClosureExpr) expression() {}

// Channel expression
type ChanExpr struct {
	Type Type `json:"type"`
}

func (e ChanExpr) expression() {}

// Operators
type BinaryOp int

const (
	OpAdd BinaryOp = iota
	OpSubtract
	OpMultiply
	OpDivide
	OpModulo
	OpAssign
	OpEqual
	OpNotEqual
	OpLessThan
	OpGreaterThan
	OpLessThanOrEqual
	OpGreaterThanOrEqual
	OpAnd
	OpOr
)

func (op BinaryOp) String() string {
	switch op {
	case OpAdd:
		return "+"
	case OpSubtract:
		return "-"
	case OpMultiply:
		return "*"
	case OpDivide:
		return "/"
	case OpModulo:
		return "%"
	case OpAssign:
		return "="
	case OpEqual:
		return "=="
	case OpNotEqual:
		return "!="
	case OpLessThan:
		return "<"
	case OpGreaterThan:
		return ">"
	case OpLessThanOrEqual:
		return "<="
	case OpGreaterThanOrEqual:
		return ">="
	case OpAnd:
		return "&&"
	case OpOr:
		return "||"
	}
	return ""
}

type UnaryOp int

const (
	OpNegate UnaryOp = iota
	OpNot
	OpPostInc
	OpPostDec
)

func (op UnaryOp) String() string {
	switch op {
	case OpNegate:
		return "-"
	case OpNot:
		return "!"
	case OpPostInc:
		return "++"
	case OpPostDec:
		return "--"
	}
	return ""
}

// JSON marshaling for interfaces
func (p *Program) MarshalJSON() ([]byte, error) {
	type Alias Program
	return json.Marshal(&struct {
		*Alias
	}{Alias: (*Alias)(p)})
}

// Note: For unmarshaling, we'd need custom logic for interfaces, but for now, assume we build AST programmatically.
