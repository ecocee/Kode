package ir

import (
	"github.com/ecocee/kode-go/pkg/ast"
)

// IR represents the intermediate representation
type IR struct {
	Program *IRProgram
	AST     ast.Program // original AST for interpretation
}

// IRProgram is the top-level IR program
type IRProgram struct {
	Functions []*IRFunction
	Globals   []*IRGlobal
}

// IRFunction represents a function in IR
type IRFunction struct {
	Name       string
	Params     []string
	ReturnType ast.Type
	Body       []*IRBlock
}

// IRBlock represents a basic block
type IRBlock struct {
	Label        string
	Instructions []IRInstruction
}

// IRInstruction represents an instruction
type IRInstruction interface {
	instruction()
}

// Concrete instructions
type IRBinaryOp struct {
	Op     ast.BinaryOp
	Left   IRValue
	Right  IRValue
	Result string
}

func (i IRBinaryOp) instruction() {}

type IRCall struct {
	Function string
	Args     []IRValue
	Result   string
}

func (i IRCall) instruction() {}

type IRReturn struct {
	Value IRValue
}

func (i IRReturn) instruction() {}

type IRJump struct {
	Label string
}

func (i IRJump) instruction() {}

type IRCondJump struct {
	Condition  IRValue
	TrueLabel  string
	FalseLabel string
}

func (i IRCondJump) instruction() {}

type IRPhi struct {
	Values []IRValue
	Result string
}

func (i IRPhi) instruction() {}

// IRValue represents a value in IR
type IRValue interface {
	value()
}

// Concrete values
type IRConstant struct {
	Type  ast.Type
	Value interface{}
}

func (v IRConstant) value() {}

type IRVariable struct {
	Name string
	Type ast.Type
}

func (v IRVariable) value() {}

type IRGlobal struct {
	Name  string
	Type  ast.Type
	Value IRValue
}

// NewIR creates a new IR
func NewIR() *IR {
	return &IR{
		Program: &IRProgram{
			Functions: []*IRFunction{},
			Globals:   []*IRGlobal{},
		},
	}
}
