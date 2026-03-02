package bytecode

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"sort"
)

// OpCode represents a bytecode operation
type OpCode byte

const (
	// Stack operations
	OpPush        OpCode = iota // Push constant onto stack
	OpPop                       // Pop from stack
	OpDup                       // Duplicate top of stack
	OpLoad                      // Load variable (arg: var index)
	OpStore                     // Store to variable (arg: var index)
	OpLoadGlobal                // Load global variable (arg: var index)
	OpStoreGlobal               // Store global variable (arg: var index)

	// Arithmetic
	OpAdd  // Add
	OpSub  // Subtract
	OpMul  // Multiply
	OpDiv  // Divide
	OpMod  // Modulo
	OpNeg  // Negate
	OpIncr // Increment
	OpDecr // Decrement

	// Bitwise
	OpBitAnd // Bitwise AND
	OpBitOr  // Bitwise OR
	OpBitXor // Bitwise XOR
	OpBitNot // Bitwise NOT
	OpBitShl // Bitwise shift left
	OpBitShr // Bitwise shift right

	// Comparison
	OpEq  // Equal
	OpNe  // Not equal
	OpLt  // Less than
	OpLte // Less than or equal
	OpGt  // Greater than
	OpGte // Greater than or equal

	// Logic
	OpAnd // Logical AND
	OpOr  // Logical OR
	OpNot // Logical NOT

	// Control flow
	OpJmp        // Jump (arg: offset)
	OpJmpIfFalse // Jump if false (arg: offset)
	OpJmpIfTrue  // Jump if true (arg: offset)

	// Function call
	OpCall        // Call function (arg: func index, arg count)
	OpReturn      // Return from function
	OpReturnValue // Return with value

	// Arrays
	OpArrayCreate  // Create array from stack values (arg: element count)
	OpArrayAccess  // Access array element (pops index, pops array, pushes element)
	OpArrayStore   // Store to array element (pops value, pops index, pops array)
	OpArrayLen     // Get array length (pops array, pushes length)
	OpMemberAccess // Member access (arg: member name; pops object, pushes member value)

	// Structs
	OpStructCreate // Create struct (arg: struct name, field count)
	OpStructField  // Access struct field (arg: field name; pops struct, pushes field value)

	// Enums
	OpEnumVariant // Create enum variant (arg: enum name, variant name)

	// Other
	OpPrint           // Print stack top
	OpInput           // Read input from user
	OpInputWithPrompt // Read input from user with prompt
	OpBreak           // Break from loop (arg: jump offset)
	OpContinue        // Continue loop (arg: jump offset)
	OpNoop            // No operation
	OpHalt            // End execution
)

// Value represents a runtime value
type Value interface{}

// SourceInfo represents source location information
type SourceInfo struct {
	File string
	Line int
}

// Instruction represents a single bytecode instruction
type Instruction struct {
	Op   OpCode
	Args []interface{}
}

// Program represents compiled bytecode
type Program struct {
	Instructions []Instruction
	Constants    []Value
	Globals      map[string]int     // Variable name -> index
	Functions    map[string]int     // Function name -> entry pc
	SourceMap    map[int]SourceInfo // PC -> source location
}

// Buffer for building bytecode
type Buffer struct {
	buf          bytes.Buffer
	instructions []Instruction
	constants    []Value
	globals      map[string]int
	functions    map[string]int
	sourceMap    map[int]SourceInfo
}

// NewBuffer creates a new bytecode buffer
func NewBuffer() *Buffer {
	return &Buffer{
		instructions: make([]Instruction, 0),
		constants:    make([]Value, 0),
		globals:      make(map[string]int),
		functions:    make(map[string]int),
		sourceMap:    make(map[int]SourceInfo),
	}
}

// Emit adds an instruction to the buffer
func (b *Buffer) Emit(op OpCode, args ...interface{}) {
	b.instructions = append(b.instructions, Instruction{
		Op:   op,
		Args: args,
	})
	// For now, don't set source info - will be updated when calls are modified
}

// EmitWithSource adds an instruction with source location
func (b *Buffer) EmitWithSource(file string, line int, op OpCode, args ...interface{}) {
	pc := len(b.instructions)
	b.instructions = append(b.instructions, Instruction{
		Op:   op,
		Args: args,
	})
	b.sourceMap[pc] = SourceInfo{File: file, Line: line}
}

// AddConstant adds a constant value and returns its index
func (b *Buffer) AddConstant(val Value) int {
	idx := len(b.constants)
	b.constants = append(b.constants, val)
	return idx
}

// AddGlobal adds a global variable and returns its index
func (b *Buffer) AddGlobal(name string) int {
	if idx, exists := b.globals[name]; exists {
		return idx
	}
	idx := len(b.globals)
	b.globals[name] = idx
	return idx
}

// RegisterFunction registers a function entry point
// Returns the PC where the function starts
func (b *Buffer) RegisterFunction(name string) int {
	pc := len(b.instructions)
	b.functions[name] = pc
	return pc
}

// GetFunctionPC gets the entry point PC for a function
func (b *Buffer) GetFunctionPC(name string) (int, bool) {
	pc, ok := b.functions[name]
	return pc, ok
}

// Build converts the buffer to a Program
func (b *Buffer) Build() *Program {
	return &Program{
		Instructions: b.instructions,
		Constants:    b.constants,
		Globals:      b.globals,
		Functions:    b.functions,
	}
}

// String returns a human-readable representation
func (p *Program) String() string {
	var s string
	s += fmt.Sprintf("Constants: %v\n", p.Constants)
	s += fmt.Sprintf("Globals: %v\n", p.Globals)
	s += fmt.Sprintf("Instructions: %d bytes\n", len(p.Instructions))
	return s
}

// Serialize converts program to bytes
func (p *Program) Serialize() ([]byte, error) {
	var buf bytes.Buffer

	// Magic number for safety
	buf.WriteString("KODE")

	// Write number of constants
	binary.Write(&buf, binary.LittleEndian, int32(len(p.Constants)))

	// Write constants
	for _, c := range p.Constants {
		switch v := c.(type) {
		case nil:
			buf.WriteByte(0) // Type: nil
		case int:
			buf.WriteByte(1) // Type: int
			binary.Write(&buf, binary.LittleEndian, int64(v))
		case float64:
			buf.WriteByte(2) // Type: float
			binary.Write(&buf, binary.LittleEndian, v)
		case bool:
			buf.WriteByte(3) // Type: bool
			if v {
				buf.WriteByte(1)
			} else {
				buf.WriteByte(0)
			}
		case string:
			buf.WriteByte(4) // Type: string
			strBytes := []byte(v)
			binary.Write(&buf, binary.LittleEndian, int32(len(strBytes)))
			buf.Write(strBytes)
		default:
			buf.WriteByte(0) // Treat unknown as nil
		}
	}

	// Write number of globals
	binary.Write(&buf, binary.LittleEndian, int32(len(p.Globals)))

	// Write globals (in sorted order for consistency)
	var globalNames []string
	for name := range p.Globals {
		globalNames = append(globalNames, name)
	}
	// Sort for deterministic serialization
	sort.Strings(globalNames)
	for _, name := range globalNames {
		idx := p.Globals[name]
		nameBytes := []byte(name)
		binary.Write(&buf, binary.LittleEndian, int32(len(nameBytes)))
		buf.Write(nameBytes)
		binary.Write(&buf, binary.LittleEndian, int32(idx))
	}

	// Write instructions (store as raw buffer for now)
	instructionData, _ := serializeInstructions(p.Instructions)
	binary.Write(&buf, binary.LittleEndian, int32(len(instructionData)))
	buf.Write(instructionData)

	return buf.Bytes(), nil
}

// Deserialize converts bytes back to a program
func Deserialize(data []byte) (*Program, error) {
	var buf = bytes.NewReader(data)

	// Check magic number
	magic := make([]byte, 4)
	buf.Read(magic)
	if string(magic) != "KODE" {
		return nil, fmt.Errorf("invalid bytecode magic number")
	}

	prog := &Program{
		Instructions: make([]Instruction, 0),
		Constants:    make([]Value, 0),
		Globals:      make(map[string]int),
	}

	// Read number of constants
	var numConstants int32
	binary.Read(buf, binary.LittleEndian, &numConstants)

	// Read constants
	for i := 0; i < int(numConstants); i++ {
		typeB := make([]byte, 1)
		buf.Read(typeB)

		switch typeB[0] {
		case 0: // nil
			prog.Constants = append(prog.Constants, nil)
		case 1: // int
			var val int64
			binary.Read(buf, binary.LittleEndian, &val)
			prog.Constants = append(prog.Constants, int(val))
		case 2: // float
			var val float64
			binary.Read(buf, binary.LittleEndian, &val)
			prog.Constants = append(prog.Constants, val)
		case 3: // bool
			b := make([]byte, 1)
			buf.Read(b)
			prog.Constants = append(prog.Constants, b[0] != 0)
		case 4: // string
			var len int32
			binary.Read(buf, binary.LittleEndian, &len)
			strBytes := make([]byte, len)
			buf.Read(strBytes)
			prog.Constants = append(prog.Constants, string(strBytes))
		}
	}

	// Read number of globals
	var numGlobals int32
	binary.Read(buf, binary.LittleEndian, &numGlobals)

	// Read globals
	for i := 0; i < int(numGlobals); i++ {
		var nameLen int32
		binary.Read(buf, binary.LittleEndian, &nameLen)
		nameBytes := make([]byte, nameLen)
		buf.Read(nameBytes)

		var idx int32
		binary.Read(buf, binary.LittleEndian, &idx)
		prog.Globals[string(nameBytes)] = int(idx)
	}

	// Read instructions
	var instrLen int32
	binary.Read(buf, binary.LittleEndian, &instrLen)
	instrBytes := make([]byte, instrLen)
	buf.Read(instrBytes)

	instrs, _ := deserializeInstructions(instrBytes)
	prog.Instructions = instrs

	return prog, nil
}

// Helper to serialize instructions
func serializeInstructions(instrs []Instruction) ([]byte, error) {
	var buf bytes.Buffer
	binary.Write(&buf, binary.LittleEndian, int32(len(instrs)))
	for _, instr := range instrs {
		buf.WriteByte(byte(instr.Op))
		// Store args count
		binary.Write(&buf, binary.LittleEndian, int32(len(instr.Args)))
		// Store args (simplified - just int/float/string)
		for _, arg := range instr.Args {
			switch v := arg.(type) {
			case int:
				buf.WriteByte(1)
				binary.Write(&buf, binary.LittleEndian, int32(v))
			case float64:
				buf.WriteByte(2)
				binary.Write(&buf, binary.LittleEndian, v)
			case string:
				buf.WriteByte(3)
				binary.Write(&buf, binary.LittleEndian, int32(len(v)))
				buf.WriteString(v)
			}
		}
	}
	return buf.Bytes(), nil
}

// Helper to deserialize instructions
func deserializeInstructions(data []byte) ([]Instruction, error) {
	buf := bytes.NewReader(data)
	var instrs []Instruction

	var count int32
	binary.Read(buf, binary.LittleEndian, &count)

	for i := 0; i < int(count); i++ {
		opByte := make([]byte, 1)
		buf.Read(opByte)
		op := OpCode(opByte[0])

		var argCount int32
		binary.Read(buf, binary.LittleEndian, &argCount)

		var args []interface{}
		for j := 0; j < int(argCount); j++ {
			typeByte := make([]byte, 1)
			buf.Read(typeByte)

			switch typeByte[0] {
			case 1: // int
				var val int32
				binary.Read(buf, binary.LittleEndian, &val)
				args = append(args, int(val))
			case 2: // float
				var val float64
				binary.Read(buf, binary.LittleEndian, &val)
				args = append(args, val)
			case 3: // string
				var len int32
				binary.Read(buf, binary.LittleEndian, &len)
				strBytes := make([]byte, len)
				buf.Read(strBytes)
				args = append(args, string(strBytes))
			}
		}

		instrs = append(instrs, Instruction{Op: op, Args: args})
	}

	return instrs, nil
}
