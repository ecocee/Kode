package parser

import (
	"fmt"
	"strings"

	"github.com/ecocee/kode-go/internal/lexer"
	"github.com/ecocee/kode-go/pkg/ast"
)

type Parser struct {
	tokens     []lexer.Token
	current    int
	filePath   string
	sourceCode string
	filePrefix string
}

func NewParser(filePath, sourceCode string) (*Parser, error) {
	lex := lexer.NewLexer(sourceCode)
	tokens, err := lex.Tokenize()
	if err != nil {
		return nil, err
	}

	filePrefix := filePath
	if filePrefix == "" {
		filePrefix = "main"
	}

	return &Parser{
		tokens:     tokens,
		current:    0,
		filePath:   filePath,
		sourceCode: sourceCode,
		filePrefix: filePrefix,
	}, nil
}

// newParserFromTokens creates a parser from an already-lexed token slice.
// Used for parsing sub-expressions inside interpolated strings.
func newParserFromTokens(tokens []lexer.Token) *Parser {
	return &Parser{
		tokens:  tokens,
		current: 0,
	}
}

func (p *Parser) Parse() ([]ast.Statement, error) {
	var statements []ast.Statement
	for !p.isAtEnd() {
		stmt, err := p.declaration()
		if err != nil {
			return nil, err
		}
		statements = append(statements, stmt)
	}
	return statements, nil
}

func (p *Parser) ParseModule() ([]ast.Statement, error) {
	var statements []ast.Statement
	for !p.isAtEnd() {
		if p.match(lexer.TokenGo, lexer.TokenFunc, lexer.TokenImport) {
			p.current-- // backtrack
			stmt, err := p.declaration()
			if err != nil {
				return nil, err
			}
			statements = append(statements, stmt)
		} else {
			p.advance()
		}
	}
	return statements, nil
}

func (p *Parser) declaration() (ast.Statement, error) {
	var stmt ast.Statement
	var err error
	if p.match(lexer.TokenExport) {
		stmt, err = p.exportDeclaration()
	} else if p.match(lexer.TokenLet) {
		stmt, err = p.letDeclaration()
	} else if p.match(lexer.TokenConst) {
		stmt, err = p.constDeclaration()
	} else if p.match(lexer.TokenFunc) {
		stmt, err = p.functionDefinition()
	} else if p.match(lexer.TokenStruct) {
		stmt, err = p.structDeclaration()
	} else if p.match(lexer.TokenEnum) {
		stmt, err = p.enumDeclaration()
	} else if p.match(lexer.TokenTrait) {
		stmt, err = p.traitDeclaration()
	} else if p.match(lexer.TokenImpl) {
		stmt, err = p.implDeclaration()
	} else if p.match(lexer.TokenService) {
		stmt, err = p.serviceDeclaration()
	} else if p.match(lexer.TokenMod) {
		stmt, err = p.moduleDeclaration()
	} else if p.match(lexer.TokenImport, lexer.TokenUse) {
		stmt, err = p.importDeclaration()
	} else if p.match(lexer.TokenSpawn) {
		stmt, err = p.spawnStatement()
	} else {
		stmt, err = p.statement()
	}
	if err != nil {
		return nil, err
	}
	// consume optional semicolon after statement
	p.match(lexer.TokenSemicolon)
	return stmt, nil
}

func (p *Parser) letDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'let' token (already consumed by match in declaration)
	name, err := p.consumeIdentifier("Expected variable name after 'let'")
	if err != nil {
		return nil, err
	}

	var typ ast.Type
	if p.match(lexer.TokenColon) {
		t, err := p.parseType()
		if err != nil {
			return nil, err
		}
		typ = t
	}

	if _, err := p.consume(lexer.TokenEqual, "Expected '=' after variable declaration"); err != nil {
		return nil, err
	}

	value, err := p.expression()
	if err != nil {
		return nil, err
	}

	stmt := ast.LetStmt{Line: line, Name: name, Type: typ, Value: value}
	return stmt, nil
}

func (p *Parser) functionDefinition() (ast.Statement, error) {
	line := p.peek().Pos.Line
	isMain := p.match(lexer.TokenMain)

	var name string
	if !isMain {
		n, err := p.consumeIdentifier("Expected function name after 'fn'")
		if err != nil {
			return nil, err
		}
		name = n
	}

	if _, err := p.consume(lexer.TokenLParen, "Expected '(' after function name"); err != nil {
		return nil, err
	}

	var params []ast.Param
	if !p.check(lexer.TokenRParen) {
		param, err := p.parseParam()
		if err != nil {
			return nil, err
		}
		params = append(params, param)

		for p.match(lexer.TokenComma) {
			param, err := p.parseParam()
			if err != nil {
				return nil, err
			}
			params = append(params, param)
		}
	}

	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after parameters"); err != nil {
		return nil, err
	}

	var returnType ast.Type
	if !p.check(lexer.TokenLBrace) && !p.check(lexer.TokenEqual) && !p.check(lexer.TokenArrow) {
		rt, err := p.parseType()
		if err != nil {
			return nil, err
		}
		returnType = rt
	}

	var body interface{}
	var isExprBody bool
	if p.match(lexer.TokenArrow) || p.match(lexer.TokenEqual) {
		// Expression body (support both => and =)
		expr, err := p.expression()
		if err != nil {
			return nil, err
		}
		body = expr
		isExprBody = true
	} else if p.match(lexer.TokenLBrace) {
		// Block body
		block, err := p.block()
		if err != nil {
			return nil, err
		}
		body = block
		isExprBody = false
	} else {
		return nil, fmt.Errorf("Expected '=>', '=' or '{' after function signature")
	}

	return ast.FunctionDefStmt{
		Line:       line,
		FilePrefix: p.filePrefix,
		IsMain:     isMain,
		Name:       name,
		Params:     params,
		ReturnType: returnType,
		Body:       body,
		IsExprBody: isExprBody,
	}, nil
}

func (p *Parser) parseParam() (ast.Param, error) {
	name, err := p.consumeIdentifier("Expected parameter name")
	if err != nil {
		return ast.Param{}, err
	}

	// Type annotation is optional — fn(x, y) and fn(x: int, y: int) both valid
	if p.match(lexer.TokenColon) {
		typ, err := p.parseType()
		if err != nil {
			return ast.Param{}, err
		}
		return ast.Param{Name: name, Type: typ}, nil
	}

	return ast.Param{Name: name}, nil
}

func (p *Parser) parseType() (ast.Type, error) {
	if p.match(lexer.TokenInt) {
		return ast.IntType{}, nil
	} else if p.match(lexer.TokenStringType) {
		return ast.StringType{}, nil
	} else if p.match(lexer.TokenBoolType) {
		return ast.BoolType{}, nil
	} else if p.match(lexer.TokenFloatType) {
		return ast.FloatType{}, nil
	} else if p.match(lexer.TokenLBracket) {
		elemType, err := p.parseType()
		if err != nil {
			return nil, err
		}
		if _, err := p.consume(lexer.TokenRBracket, "Expected ']' after array type"); err != nil {
			return nil, err
		}
		return ast.ArrayType{ElementType: elemType}, nil
	}
	return nil, fmt.Errorf("Expected type")
}

func (p *Parser) constDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'const' token
	name, err := p.consumeIdentifier("Expected constant name after 'const'")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenColon, "Expected ':' after constant name"); err != nil {
		return nil, err
	}

	typ, err := p.parseType()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenEqual, "Expected '=' after constant type"); err != nil {
		return nil, err
	}

	value, err := p.expression()
	if err != nil {
		return nil, err
	}

	// Semicolon is now optional
	p.match(lexer.TokenSemicolon)

	return ast.ConstDeclStmt{Line: line, Name: name, Type: typ, Value: value}, nil
}

func (p *Parser) structDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'struct' token
	name, err := p.consumeIdentifier("Expected struct name")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after struct name"); err != nil {
		return nil, err
	}

	var fields []ast.Field
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		fieldName, err := p.consumeIdentifier("Expected field name")
		if err != nil {
			return nil, err
		}

		if _, err := p.consume(lexer.TokenColon, "Expected ':' after field name"); err != nil {
			return nil, err
		}

		fieldType, err := p.parseType()
		if err != nil {
			return nil, err
		}

		fields = append(fields, ast.Field{Name: fieldName, Type: fieldType})

		if !p.match(lexer.TokenComma) {
			break
		}
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after struct fields"); err != nil {
		return nil, err
	}

	return ast.StructDeclStmt{Line: line, Name: name, Fields: fields}, nil
}

func (p *Parser) enumDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'enum' token
	name, err := p.consumeIdentifier("Expected enum name")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after enum name"); err != nil {
		return nil, err
	}

	var variants []string
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		variant, err := p.consumeIdentifier("Expected variant name")
		if err != nil {
			return nil, err
		}
		variants = append(variants, variant)

		if !p.match(lexer.TokenComma) {
			break
		}
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after enum variants"); err != nil {
		return nil, err
	}

	return ast.EnumDeclStmt{Line: line, Name: name, Variants: variants}, nil
}

func (p *Parser) traitDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'trait' token
	name, err := p.consumeIdentifier("Expected trait name")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after trait name"); err != nil {
		return nil, err
	}

	var signatures []ast.FunctionSignature
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		if _, err := p.consume(lexer.TokenFunc, "Expected 'func' in trait"); err != nil {
			return nil, err
		}

		fnName, err := p.consumeIdentifier("Expected function name")
		if err != nil {
			return nil, err
		}

		if _, err := p.consume(lexer.TokenLParen, "Expected '(' after function name"); err != nil {
			return nil, err
		}

		var params []ast.Param
		if !p.check(lexer.TokenRParen) {
			param, err := p.parseParam()
			if err != nil {
				return nil, err
			}
			params = append(params, param)

			for p.match(lexer.TokenComma) {
				param, err := p.parseParam()
				if err != nil {
					return nil, err
				}
				params = append(params, param)
			}
		}

		if _, err := p.consume(lexer.TokenRParen, "Expected ')' after parameters"); err != nil {
			return nil, err
		}

		var returnType ast.Type
		if p.match(lexer.TokenArrow) { // Assume -> token
			rt, err := p.parseType()
			if err != nil {
				return nil, err
			}
			returnType = rt
		}

		signatures = append(signatures, ast.FunctionSignature{
			Name:       fnName,
			Params:     params,
			ReturnType: returnType,
		})

		if !p.match(lexer.TokenComma) {
			break
		}
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after trait"); err != nil {
		return nil, err
	}

	return ast.TraitDeclStmt{Line: line, Name: name, Signatures: signatures}, nil
}

func (p *Parser) implDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'impl' token
	trait, err := p.consumeIdentifier("Expected trait name")
	if err != nil {
		return nil, err
	}

	typ, err := p.consumeIdentifier("Expected type name")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after impl"); err != nil {
		return nil, err
	}

	var methods []ast.FunctionDefStmt
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		method, err := p.functionDefinition()
		if err != nil {
			return nil, err
		}
		if fn, ok := method.(ast.FunctionDefStmt); ok {
			methods = append(methods, fn)
		}
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after impl"); err != nil {
		return nil, err
	}

	return ast.ImplDeclStmt{Line: line, Trait: trait, Type: typ, Methods: methods}, nil
}

func (p *Parser) serviceDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'service' token
	name, err := p.consumeIdentifier("Expected service name")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after service name"); err != nil {
		return nil, err
	}

	var routes []ast.RouteStmt
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		if _, err := p.consume(lexer.TokenRoute, "Expected 'route' in service"); err != nil {
			return nil, err
		}

		method, err := p.consumeIdentifier("Expected HTTP method")
		if err != nil {
			return nil, err
		}

		path, err := p.consumeString("Expected path")
		if err != nil {
			return nil, err
		}

		if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after route"); err != nil {
			return nil, err
		}

		body, err := p.block()
		if err != nil {
			return nil, err
		}

		routes = append(routes, ast.RouteStmt{Method: method, Path: path, Body: body})
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after service"); err != nil {
		return nil, err
	}

	return ast.ServiceDeclStmt{Line: line, Name: name, Routes: routes}, nil
}

func (p *Parser) moduleDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'mod' token
	name, err := p.consumeIdentifier("Expected module name")
	if err != nil {
		return nil, err
	}

	return ast.ModuleDeclStmt{Line: line, Name: name}, nil
}

// importDeclaration handles: import "path", import "path" as alias, import { items } from "path"
func (p *Parser) importDeclaration() (ast.Statement, error) {
	startPos := p.peek().Pos

	// Handle named imports: import { a, b } from "module"
	if p.check(lexer.TokenLBrace) {
		p.advance() // consume {
		var items []string
		for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
			token := p.advance()
			if token.Kind != lexer.TokenIdentifier {
				return nil, p.errorf("Invalid import item", token.Pos,
					"Each import item must be an identifier")
			}
			if val, ok := token.Value.(string); ok {
				items = append(items, val)
			}

			if !p.check(lexer.TokenRBrace) {
				if !p.match(lexer.TokenComma) {
					return nil, p.errorf("Invalid import list", p.peek().Pos,
						"Expected ',' or '}' in import list")
				}
			}
		}
		if _, err := p.consume(lexer.TokenRBrace, "Unclosed import list: Expected '}' to close import items"); err != nil {
			return nil, p.errorf("Unclosed import list", startPos, err.Error())
		}

		// Expect "from" keyword
		if !p.match(lexer.TokenFrom) {
			current := p.peek()
			return nil, p.errorf("Missing 'from' keyword", current.Pos,
				fmt.Sprintf("Expected 'from' after import items, but found '%v'. Example: import { item } from \"module\"", current.Value))
		}

		// Get module path as string
		token := p.advance()
		if token.Kind != lexer.TokenString {
			return nil, p.errorf("Invalid module path", token.Pos,
				"Module path must be a string literal. Example: from \"module_name\"")
		}
		path, ok := token.Value.(string)
		if !ok {
			return nil, p.errorf("Invalid module path value", token.Pos,
				"Could not parse module path as string")
		}

		// Semicolon is now optional
		p.match(lexer.TokenSemicolon)

		return ast.ImportStmt{Line: startPos.Line, Path: path, Items: items, IsNamed: true}, nil
	}

	// Handle: import "path" or import "path" as alias
	token := p.advance()
	if token.Kind != lexer.TokenString {
		return nil, p.errorf("Expected module path", token.Pos,
			"Import must specify a module path as a string. Example: import \"module_name\"")
	}
	path, ok := token.Value.(string)
	if !ok {
		return nil, p.errorf("Invalid module path", token.Pos,
			"Could not parse module path as string")
	}

	var alias string
	if p.match(lexer.TokenAs) {
		aliasToken := p.advance()
		if aliasToken.Kind != lexer.TokenIdentifier {
			return nil, p.errorf("Invalid alias", aliasToken.Pos,
				"Alias must be an identifier. Example: import \"module\" as m")
		}
		if val, ok := aliasToken.Value.(string); ok {
			alias = val
		}
	}

	// Semicolon is now optional
	p.match(lexer.TokenSemicolon)

	return ast.ImportStmt{Line: startPos.Line, Path: path, Alias: alias, IsNamed: false}, nil
}

func (p *Parser) exportDeclaration() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'export' token
	var stmt ast.Statement
	var err error

	if p.match(lexer.TokenFunc) {
		stmt, err = p.functionDefinition()
	} else if p.match(lexer.TokenLet) {
		stmt, err = p.letDeclaration()
	} else if p.match(lexer.TokenConst) {
		stmt, err = p.constDeclaration()
	} else if p.match(lexer.TokenStruct) {
		stmt, err = p.structDeclaration()
	} else if p.match(lexer.TokenEnum) {
		stmt, err = p.enumDeclaration()
	} else {
		current := p.peek().Value
		return nil, p.errorf("Invalid export target", p.peek().Pos,
			fmt.Sprintf("export must be followed by 'func', 'let', 'const', 'struct', or 'enum', but found '%s'. Examples: export func foo() { }, export let x = 0", current))
	}

	if err != nil {
		return nil, err
	}

	return ast.ExportStmt{Line: line, Statement: stmt, IsBlock: false}, nil
}

func (p *Parser) spawnStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'spawn' token
	call, err := p.expression()
	if err != nil {
		return nil, err
	}

	// Semicolon is now optional
	p.match(lexer.TokenSemicolon)

	return ast.SpawnStmt{Line: line, Call: call}, nil
}

func (p *Parser) statement() (ast.Statement, error) {
	if p.match(lexer.TokenLBrace) {
		line := p.previous().Pos.Line
		stmts, err := p.block()
		if err != nil {
			return nil, err
		}
		return ast.BlockStmt{Line: line, Statements: stmts}, nil
	} else if p.match(lexer.TokenIf) {
		return p.ifStatement()
	} else if p.match(lexer.TokenWhile) {
		return p.whileStatement()
	} else if p.match(lexer.TokenFor) {
		return p.forStatement()
	} else if p.match(lexer.TokenReturn) {
		return p.returnStatement()
	} else if p.match(lexer.TokenBreak) {
		return ast.BreakStmt{Line: p.previous().Pos.Line}, nil
	} else if p.match(lexer.TokenContinue) {
		return ast.ContinueStmt{Line: p.previous().Pos.Line}, nil
	} else if p.match(lexer.TokenDefer) {
		return p.deferStatement()
	} else if p.match(lexer.TokenMatch) {
		return p.matchStatement()
	} else if p.match(lexer.TokenTry) {
		return p.tryStatement()
	} else if p.check(lexer.TokenIdentifier) && p.current+1 < len(p.tokens) && p.tokens[p.current+1].Kind == lexer.TokenColon {
		// Handle name : type = expression
		line := p.peek().Pos.Line
		name, err := p.consumeIdentifier("Expected identifier")
		if err != nil {
			return nil, err
		}
		if _, err := p.consume(lexer.TokenColon, "Expected :"); err != nil {
			return nil, err
		}
		typ, err := p.parseType()
		if err != nil {
			return nil, err
		}
		if _, err := p.consume(lexer.TokenEqual, "Expected ="); err != nil {
			return nil, err
		}
		value, err := p.expression()
		if err != nil {
			return nil, err
		}
		return ast.LetStmt{
			Line:  line,
			Name:  name,
			Type:  typ,
			Value: value,
		}, nil
	} else if p.check(lexer.TokenIdentifier) && p.current+1 < len(p.tokens) && p.tokens[p.current+1].Kind == lexer.TokenWalrus {
		// Handle name := expression
		line := p.peek().Pos.Line
		name, err := p.consumeIdentifier("Expected identifier")
		if err != nil {
			return nil, err
		}
		if _, err := p.consume(lexer.TokenWalrus, "Expected :="); err != nil {
			return nil, err
		}
		value, err := p.expression()
		if err != nil {
			return nil, err
		}
		stmt := ast.LetStmt{
			Line:  line,
			Name:  name,
			Type:  nil, // Inferred
			Value: value,
		}
		return stmt, nil
	}
	return p.expressionStatement()
}

func (p *Parser) returnStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'return' token
	var value ast.Expression
	if !p.isAtEnd() && !p.check(lexer.TokenRBrace) && !p.check(lexer.TokenSemicolon) {
		v, err := p.expression()
		if err != nil {
			return nil, err
		}
		value = v
	}

	return ast.ReturnStmt{Line: line, Value: value}, nil
}

func (p *Parser) block() ([]ast.Statement, error) {
	var statements []ast.Statement
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		stmt, err := p.declaration()
		if err != nil {
			return nil, err
		}
		statements = append(statements, stmt)
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after block"); err != nil {
		return nil, err
	}

	return statements, nil
}

func (p *Parser) ifStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'if' token
	if _, err := p.consume(lexer.TokenLParen, "Expected '(' after 'if'"); err != nil {
		return nil, err
	}

	condition, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after if condition"); err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before if body"); err != nil {
		return nil, err
	}

	thenBranch, err := p.block()
	if err != nil {
		return nil, err
	}

	var elseBranch []ast.Statement
	if p.match(lexer.TokenElse) {
		if p.match(lexer.TokenIf) {
			elseIf, err := p.ifStatement()
			if err != nil {
				return nil, err
			}
			elseBranch = []ast.Statement{elseIf}
		} else {
			if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before else body"); err != nil {
				return nil, err
			}
			eb, err := p.block()
			if err != nil {
				return nil, err
			}
			elseBranch = eb
		}
	}

	return ast.IfStmt{
		Line:       line,
		Condition:  condition,
		ThenBranch: thenBranch,
		ElseBranch: elseBranch,
	}, nil
}

func (p *Parser) whileStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'while' token
	if _, err := p.consume(lexer.TokenLParen, "Expected '(' after 'while'"); err != nil {
		return nil, err
	}

	condition, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after while condition"); err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before while body"); err != nil {
		return nil, err
	}

	body, err := p.block()
	if err != nil {
		return nil, err
	}

	return ast.WhileStmt{Line: line, Condition: condition, Body: body}, nil
}

func (p *Parser) forStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'for' token

	// Check for for-in loop: for identifier in iterable { body }
	if p.check(lexer.TokenIdentifier) && p.current+1 < len(p.tokens) && p.tokens[p.current+1].Kind == lexer.TokenIn {
		return p.forInStatement(line)
	}

	if _, err := p.consume(lexer.TokenLParen, "Expected '(' after 'for'"); err != nil {
		return nil, err
	}

	var init interface{}
	if p.match(lexer.TokenLet) {
		i, err := p.letDeclaration()
		if err != nil {
			return nil, err
		}
		init = i
	} else if !p.check(lexer.TokenSemicolon) {
		i, err := p.expression()
		if err != nil {
			return nil, err
		}
		init = i
	}
	// Semicolon is now optional in for loops, but we still expect the structure
	if !p.match(lexer.TokenSemicolon) && !p.check(lexer.TokenRParen) {
		return nil, p.errorf("Syntax Error", p.peek().Pos,
			fmt.Sprintf("Expected ';' after for loop init, got '%v'. For loop syntax: for(init; cond; incr)", p.peek().Value))
	}

	var cond ast.Expression
	if !p.check(lexer.TokenSemicolon) && !p.check(lexer.TokenRParen) {
		c, err := p.expression()
		if err != nil {
			return nil, err
		}
		cond = c
	}
	if !p.match(lexer.TokenSemicolon) && !p.check(lexer.TokenRParen) {
		return nil, p.errorf("Syntax Error", p.peek().Pos,
			fmt.Sprintf("Expected ';' after for loop condition, got '%v'. For loop syntax: for(init; cond; incr)", p.peek().Value))
	}

	var incr ast.Expression
	if !p.check(lexer.TokenRParen) {
		i, err := p.expression()
		if err != nil {
			return nil, err
		}
		incr = i
	}
	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after incr"); err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before for body"); err != nil {
		return nil, err
	}

	body, err := p.block()
	if err != nil {
		return nil, err
	}

	return ast.ForStmt{
		Line:      line,
		Init:      init,
		Condition: cond,
		Incr:      incr,
		Body:      body,
	}, nil
}

func (p *Parser) forInStatement(line int) (ast.Statement, error) {
	// Parse: for identifier in iterable { body }
	varName, err := p.consumeIdentifier("Expected variable name after 'for'")
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenIn, "Expected 'in' after variable name in for-in loop"); err != nil {
		return nil, err
	}

	iterable, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before for-in body"); err != nil {
		return nil, err
	}

	body, err := p.block()
	if err != nil {
		return nil, err
	}

	return ast.ForInStmt{
		Line:     line,
		VarName:  varName,
		Iterable: iterable,
		Body:     body,
	}, nil
}

func (p *Parser) deferStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'defer' token
	call, err := p.expression()
	if err != nil {
		return nil, err
	}

	// Semicolon is now optional
	p.match(lexer.TokenSemicolon)

	return ast.DeferStmt{Line: line, Call: call}, nil
}

func (p *Parser) tryStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'try' token

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after 'try'"); err != nil {
		return nil, err
	}

	body, err := p.block()
	if err != nil {
		return nil, err
	}

	var catchBody []ast.Statement
	if p.match(lexer.TokenCatch) {
		// catch can optionally have a variable name: catch(e) { }  or just catch { }
		if p.match(lexer.TokenLParen) {
			// Parse optional catch variable
			p.consumeIdentifier("Expected catch variable name")
			if _, err := p.consume(lexer.TokenRParen, "Expected ')' after catch variable"); err != nil {
				return nil, err
			}
		}
		if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after 'catch'"); err != nil {
			return nil, err
		}
		catchBody, err = p.block()
		if err != nil {
			return nil, err
		}
	}

	return ast.TryStmt{Line: line, Body: body, Catch: catchBody}, nil
}

func (p *Parser) matchStatement() (ast.Statement, error) {
	line := p.previous().Pos.Line // 'match' token
	expr, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' after match expression"); err != nil {
		return nil, err
	}

	var cases []ast.MatchCase
	for !p.check(lexer.TokenRBrace) && !p.isAtEnd() {
		pat, err := p.pattern()
		if err != nil {
			return nil, err
		}

		if _, err := p.consume(lexer.TokenArrow, "Expected '=>' after pattern"); err != nil {
			return nil, err
		}

		body, err := p.expression()
		if err != nil {
			return nil, err
		}

		cases = append(cases, ast.MatchCase{Pattern: pat, Body: body})

		// Allow optional comma between cases; newline-separated is also fine
		p.match(lexer.TokenComma)
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after match"); err != nil {
		return nil, err
	}

	return ast.MatchStmt{Line: line, Expr: expr, Cases: cases}, nil
}

func (p *Parser) pattern() (ast.Pattern, error) {
	if p.match(lexer.TokenUnderscore) {
		return ast.WildcardPattern{}, nil
	} else if p.check(lexer.TokenIdentifier) {
		name, err := p.consumeIdentifier("Expected identifier in pattern")
		if err != nil {
			return nil, err
		}
		return ast.IdentifierPattern{Name: name}, nil
	} else if p.match(lexer.TokenString, lexer.TokenNumber, lexer.TokenTrue, lexer.TokenFalse) {
		return ast.LiteralPattern{Value: p.previous().Value}, nil
	}
	return nil, fmt.Errorf("Unexpected token in pattern: %v", p.peek().Kind)
}

func (p *Parser) expressionStatement() (ast.Statement, error) {
	line := p.peek().Pos.Line
	expr, err := p.expression()
	if err != nil {
		return nil, err
	}

	// if the expression is a binary assign with an identifier on the left, treat
	// it as an assignment statement rather than a comparison (assignment
	// operator is parsed as OpAssign).
	if bin, ok := expr.(ast.BinaryExpr); ok && bin.Op == ast.OpAssign {
		if id, ok := bin.Left.(ast.IdentifierExpr); ok {
			return ast.AssignStmt{Line: bin.Line, Name: id.Name, Value: bin.Right}, nil
		}
	}

	return ast.ExprStmt{Line: line, Expr: expr}, nil
}

// Expression parsing
func (p *Parser) expression() (ast.Expression, error) {
	return p.assignment()
}

func (p *Parser) assignment() (ast.Expression, error) {
	expr, err := p.logicOr()
	if err != nil {
		return nil, err
	}

	if p.match(lexer.TokenEqual) {
		if id, ok := expr.(ast.IdentifierExpr); ok {
			value, err := p.assignment()
			if err != nil {
				return nil, err
			}
			return ast.BinaryExpr{
				Line:  p.previous().Pos.Line,
				Left:  ast.IdentifierExpr{Name: id.Name},
				Op:    ast.OpAssign,
				Right: value,
			}, nil
		} else if aa, ok := expr.(ast.ArrayAccessExpr); ok {
			value, err := p.assignment()
			if err != nil {
				return nil, err
			}
			return ast.CallExpr{
				Callee:    ast.IdentifierExpr{Name: "__array_assign"},
				Arguments: []ast.Expression{aa.Array, aa.Index, value},
			}, nil
		}
		return nil, fmt.Errorf("Invalid assignment target")
	}

	// Compound assignment operators: +=, -=, *=, /=, %=
	var compoundOp ast.BinaryOp
	isCompound := false
	if p.match(lexer.TokenPlusEq) {
		compoundOp = ast.OpAdd
		isCompound = true
	} else if p.match(lexer.TokenMinusEq) {
		compoundOp = ast.OpSubtract
		isCompound = true
	} else if p.match(lexer.TokenStarEq) {
		compoundOp = ast.OpMultiply
		isCompound = true
	} else if p.match(lexer.TokenSlashEq) {
		compoundOp = ast.OpDivide
		isCompound = true
	} else if p.match(lexer.TokenPercentEq) {
		compoundOp = ast.OpModulo
		isCompound = true
	}
	if isCompound {
		if id, ok := expr.(ast.IdentifierExpr); ok {
			value, err := p.assignment()
			if err != nil {
				return nil, err
			}
			// Desugar: a op= b  =>  a = a op b
			return ast.BinaryExpr{
				Line: p.previous().Pos.Line,
				Left: id,
				Op:   ast.OpAssign,
				Right: ast.BinaryExpr{
					Line:  p.previous().Pos.Line,
					Left:  id,
					Op:    compoundOp,
					Right: value,
				},
			}, nil
		}
		return nil, fmt.Errorf("Invalid compound assignment target")
	}

	return expr, nil
}

func (p *Parser) logicOr() (ast.Expression, error) {
	expr, err := p.logicAnd()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenOr) {
		line := p.previous().Pos.Line
		right, err := p.logicAnd()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: ast.OpOr, Right: right}
	}

	return expr, nil
}

func (p *Parser) logicAnd() (ast.Expression, error) {
	expr, err := p.bitwiseOr()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenAnd) {
		line := p.previous().Pos.Line
		right, err := p.bitwiseOr()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: ast.OpAnd, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseOr() (ast.Expression, error) {
	expr, err := p.bitwiseXor()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitOr) {
		line := p.previous().Pos.Line
		right, err := p.bitwiseXor()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: ast.OpBitOr, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseXor() (ast.Expression, error) {
	expr, err := p.bitwiseAnd()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitXor) {
		line := p.previous().Pos.Line
		right, err := p.bitwiseAnd()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: ast.OpBitXor, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseAnd() (ast.Expression, error) {
	expr, err := p.equality()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitAnd) {
		line := p.previous().Pos.Line
		right, err := p.equality()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: ast.OpBitAnd, Right: right}
	}

	return expr, nil
}

func (p *Parser) equality() (ast.Expression, error) {
	expr, err := p.comparison()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenEqualEqual, lexer.TokenNotEqual) {
		op := ast.OpEqual
		if p.previous().Kind == lexer.TokenNotEqual {
			op = ast.OpNotEqual
		}
		line := p.previous().Pos.Line
		right, err := p.comparison()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) comparison() (ast.Expression, error) {
	expr, err := p.shift()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenLessThan, lexer.TokenGreaterThan, lexer.TokenLessThanOrEqual, lexer.TokenGreaterThanOrEqual) {
		op := ast.OpLessThan
		switch p.previous().Kind {
		case lexer.TokenGreaterThan:
			op = ast.OpGreaterThan
		case lexer.TokenLessThanOrEqual:
			op = ast.OpLessThanOrEqual
		case lexer.TokenGreaterThanOrEqual:
			op = ast.OpGreaterThanOrEqual
		}
		line := p.previous().Pos.Line
		right, err := p.shift()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) shift() (ast.Expression, error) {
	expr, err := p.term()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitShl, lexer.TokenBitShr) {
		op := ast.OpBitShl
		if p.previous().Kind == lexer.TokenBitShr {
			op = ast.OpBitShr
		}
		line := p.previous().Pos.Line
		right, err := p.term()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) term() (ast.Expression, error) {
	expr, err := p.factor()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenPlus, lexer.TokenMinus) {
		op := ast.OpAdd
		if p.previous().Kind == lexer.TokenMinus {
			op = ast.OpSubtract
		}
		line := p.previous().Pos.Line
		right, err := p.factor()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) factor() (ast.Expression, error) {
	expr, err := p.unary()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenStar, lexer.TokenSlash, lexer.TokenPercent) {
		op := ast.OpMultiply
		switch p.previous().Kind {
		case lexer.TokenSlash:
			op = ast.OpDivide
		case lexer.TokenPercent:
			op = ast.OpModulo
		}
		line := p.previous().Pos.Line
		right, err := p.unary()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Line: line, Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) unary() (ast.Expression, error) {
	// Prefix ++ and --
	if p.match(lexer.TokenPlusPlus) {
		line := p.previous().Pos.Line
		expr, err := p.call()
		if err != nil {
			return nil, err
		}
		return ast.UnaryExpr{Line: line, Op: ast.OpPreInc, Expr: expr}, nil
	}
	if p.match(lexer.TokenMinusMinus) {
		line := p.previous().Pos.Line
		expr, err := p.call()
		if err != nil {
			return nil, err
		}
		return ast.UnaryExpr{Line: line, Op: ast.OpPreDec, Expr: expr}, nil
	}
	if p.match(lexer.TokenMinus, lexer.TokenNot, lexer.TokenBitNot) {
		line := p.previous().Pos.Line
		op := ast.OpNegate
		if p.previous().Kind == lexer.TokenNot {
			op = ast.OpNot
		} else if p.previous().Kind == lexer.TokenBitNot {
			op = ast.OpBitNot
		}
		right, err := p.unary()
		if err != nil {
			return nil, err
		}
		return ast.UnaryExpr{Line: line, Op: op, Expr: right}, nil
	}

	return p.call()
}

func (p *Parser) call() (ast.Expression, error) {
	expr, err := p.primary()
	if err != nil {
		return nil, err
	}

	for {
		if p.match(lexer.TokenLParen) {
			expr, err = p.finishCall(expr)
			if err != nil {
				return nil, err
			}
		} else if p.match(lexer.TokenLBracket) {
			index, err := p.expression()
			if err != nil {
				return nil, err
			}
			if _, err := p.consume(lexer.TokenRBracket, "Expected ']' after array index"); err != nil {
				return nil, err
			}
			expr = ast.ArrayAccessExpr{Line: p.previous().Pos.Line, Array: expr, Index: index}
		} else if p.match(lexer.TokenDot) {
			// Member access: obj.member or arr.len()
			if !p.check(lexer.TokenIdentifier) {
				return nil, fmt.Errorf("Expected member name after '.'")
			}
			if val, ok := p.advance().Value.(string); ok {
				expr = ast.MemberAccessExpr{Line: p.previous().Pos.Line, Object: expr, Member: val}
			} else {
				return nil, fmt.Errorf("Invalid member name")
			}
		} else if p.checkStructLiteralStart() {
			lbraceLine := p.peek().Pos.Line
			p.advance() // consume {
			// Struct literal: StructName { field1: value1, field2: value2 }
			if idExpr, ok := expr.(ast.IdentifierExpr); ok {
				fields := make(map[string]ast.Expression)
				if !p.check(lexer.TokenRBrace) {
					fieldName, err := p.consumeIdentifier("Expected field name in struct literal")
					if err != nil {
						return nil, err
					}
					if _, err := p.consume(lexer.TokenColon, "Expected ':' after field name"); err != nil {
						return nil, err
					}
					fieldVal, err := p.expression()
					if err != nil {
						return nil, err
					}
					fields[fieldName] = fieldVal

					for p.match(lexer.TokenComma) {
						fieldName, err := p.consumeIdentifier("Expected field name in struct literal")
						if err != nil {
							return nil, err
						}
						if _, err := p.consume(lexer.TokenColon, "Expected ':' after field name"); err != nil {
							return nil, err
						}
						fieldVal, err := p.expression()
						if err != nil {
							return nil, err
						}
						fields[fieldName] = fieldVal
					}
				}
				if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after struct fields"); err != nil {
					return nil, err
				}
				expr = ast.StructLiteralExpr{Line: lbraceLine, StructName: idExpr.Name, Fields: fields}
			} else {
				return nil, fmt.Errorf("Struct literal must have a struct name")
			}
		} else {
			break
		}
	}

	return expr, nil
}

func (p *Parser) finishCall(callee ast.Expression) (ast.Expression, error) {
	var arguments []ast.Expression
	if !p.check(lexer.TokenRParen) {
		arg, err := p.expression()
		if err != nil {
			return nil, err
		}
		arguments = append(arguments, arg)

		for p.match(lexer.TokenComma) {
			arg, err := p.expression()
			if err != nil {
				return nil, err
			}
			arguments = append(arguments, arg)
		}
	}

	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after arguments"); err != nil {
		return nil, err
	}

	return ast.CallExpr{Line: p.previous().Pos.Line, Callee: callee, Arguments: arguments}, nil
}

func (p *Parser) primary() (ast.Expression, error) {
	var expr ast.Expression
	if p.match(lexer.TokenNumber) {
		if val, ok := p.previous().Value.(int64); ok {
			expr = ast.NumberExpr{Line: p.previous().Pos.Line, Value: val}
		}
	} else if p.match(lexer.TokenFloat) {
		if val, ok := p.previous().Value.(float64); ok {
			expr = ast.FloatExpr{Line: p.previous().Pos.Line, Value: val}
		}
	} else if p.match(lexer.TokenBool) {
		if val, ok := p.previous().Value.(bool); ok {
			expr = ast.BoolExpr{Line: p.previous().Pos.Line, Value: val}
		}
	} else if p.match(lexer.TokenString) {
		if val, ok := p.previous().Value.(string); ok {
			expr = ast.StringExpr{Line: p.previous().Pos.Line, Value: val}
		}
	} else if p.match(lexer.TokenStringInterp) {
		// Interpolated string: "Hello, ${name}!"
		line := p.previous().Pos.Line
		rawParts, ok := p.previous().Value.([]lexer.StringInterpPart)
		if !ok {
			return nil, fmt.Errorf("internal: invalid interpolated string token")
		}
		var astParts []ast.StringInterpPart
		for _, rp := range rawParts {
			if !rp.IsExpr {
				astParts = append(astParts, ast.StringInterpPart{IsExpr: false, Literal: rp.Content})
			} else {
				// Sub-lex and sub-parse the embedded expression
				subLex := lexer.NewLexer(rp.Content)
				subTokens, err := subLex.Tokenize()
				if err != nil {
					return nil, fmt.Errorf("error in interpolated expression: %v", err)
				}
				subParser := newParserFromTokens(subTokens)
				subExpr, err := subParser.expression()
				if err != nil {
					return nil, fmt.Errorf("error parsing interpolated expression '${%s}': %v", rp.Content, err)
				}
				astParts = append(astParts, ast.StringInterpPart{IsExpr: true, Expr: subExpr})
			}
		}
		expr = ast.StringInterpExpr{Line: line, Parts: astParts}
	} else if p.match(lexer.TokenIdentifier) {
		if val, ok := p.previous().Value.(string); ok {
			expr = ast.IdentifierExpr{Line: p.previous().Pos.Line, Name: val}
		}
	} else if p.match(lexer.TokenLBracket) {
		var err error
		expr, err = p.arrayLiteral()
		if err != nil {
			return nil, err
		}
	} else if p.match(lexer.TokenLParen) {
		var err error
		expr, err = p.expression()
		if err != nil {
			return nil, err
		}
		if _, err := p.consume(lexer.TokenRParen, "Expected ')' after expression"); err != nil {
			return nil, err
		}
	} else if p.match(lexer.TokenFunc) {
		var err error
		expr, err = p.closure()
		if err != nil {
			return nil, err
		}
	} else if p.match(lexer.TokenNil) {
		expr = ast.NilExpr{Line: p.previous().Pos.Line}
	} else if p.match(lexer.TokenInt) {
		// Allow type names as callable identifiers: int("42"), float("3.14"), etc.
		expr = ast.IdentifierExpr{Line: p.previous().Pos.Line, Name: "int"}
	} else if p.match(lexer.TokenStringType) {
		expr = ast.IdentifierExpr{Line: p.previous().Pos.Line, Name: "string"}
	} else if p.match(lexer.TokenBoolType) {
		expr = ast.IdentifierExpr{Line: p.previous().Pos.Line, Name: "bool"}
	} else if p.match(lexer.TokenFloatType) {
		expr = ast.IdentifierExpr{Line: p.previous().Pos.Line, Name: "float"}
	} else if p.match(lexer.TokenChan) {
		typ, err := p.parseType()
		if err != nil {
			return nil, err
		}
		expr = ast.ChanExpr{Line: p.previous().Pos.Line, Type: typ}
	} else {
		tok := p.peek()
		return nil, fmt.Errorf("Expected expression at line %d (got token kind: %v)", tok.Pos.Line, tok.Kind)
	}

	if p.match(lexer.TokenPlusPlus) {
		expr = ast.UnaryExpr{Line: p.previous().Pos.Line, Op: ast.OpPostInc, Expr: expr}
	} else if p.match(lexer.TokenMinusMinus) {
		expr = ast.UnaryExpr{Line: p.previous().Pos.Line, Op: ast.OpPostDec, Expr: expr}
	}

	return expr, nil
}

func (p *Parser) arrayLiteral() (ast.Expression, error) {
	var elements []ast.Expression
	if !p.check(lexer.TokenRBracket) {
		elem, err := p.expression()
		if err != nil {
			return nil, err
		}
		elements = append(elements, elem)

		for p.match(lexer.TokenComma) {
			elem, err := p.expression()
			if err != nil {
				return nil, err
			}
			elements = append(elements, elem)
		}
	}

	if _, err := p.consume(lexer.TokenRBracket, "Expected ']' after array elements"); err != nil {
		return nil, err
	}

	return ast.ArrayExpr{Line: p.previous().Pos.Line, Elements: elements}, nil
}

func (p *Parser) closure() (ast.Expression, error) {
	line := p.previous().Pos.Line // The 'fn' token
	if _, err := p.consume(lexer.TokenLParen, "Expected '(' after 'fn' in closure"); err != nil {
		return nil, err
	}

	var params []ast.Param
	if !p.check(lexer.TokenRParen) {
		param, err := p.parseParam()
		if err != nil {
			return nil, err
		}
		params = append(params, param)

		for p.match(lexer.TokenComma) {
			param, err := p.parseParam()
			if err != nil {
				return nil, err
			}
			params = append(params, param)
		}
	}

	if _, err := p.consume(lexer.TokenRParen, "Expected ')' after closure parameters"); err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenLBrace, "Expected '{' before closure body"); err != nil {
		return nil, err
	}

	body, err := p.block()
	if err != nil {
		return nil, err
	}

	return ast.ClosureExpr{Line: line, Params: params, Body: body}, nil
}

// Helper methods
func (p *Parser) match(kinds ...lexer.TokenKind) bool {
	for _, kind := range kinds {
		if p.check(kind) {
			p.advance()
			return true
		}
	}
	return false
}

func (p *Parser) check(kind lexer.TokenKind) bool {
	if p.isAtEnd() {
		return false
	}
	return p.peek().Kind == kind
}

func (p *Parser) advance() lexer.Token {
	if !p.isAtEnd() {
		p.current++
	}
	return p.previous()
}

func (p *Parser) isAtEnd() bool {
	return p.current >= len(p.tokens) || p.peek().Kind == lexer.TokenEOF
}

// checkStructLiteralStart returns true if the current token is '{' and it looks
// like a struct literal (either empty '{}' or 'field: value' pairs), not a
// block body (e.g., match body '{ 1 => ... }').
func (p *Parser) checkStructLiteralStart() bool {
	if !p.check(lexer.TokenLBrace) {
		return false
	}
	next := p.current + 1
	// Empty struct: Name {}
	if next < len(p.tokens) && p.tokens[next].Kind == lexer.TokenRBrace {
		return true
	}
	// Struct literal: Name { field: value, ... }
	// Lookahead two tokens: identifier then colon
	if next+1 < len(p.tokens) {
		return p.tokens[next].Kind == lexer.TokenIdentifier &&
			p.tokens[next+1].Kind == lexer.TokenColon
	}
	return false
}

func (p *Parser) peek() lexer.Token {
	return p.tokens[p.current]
}

func (p *Parser) previous() lexer.Token {
	return p.tokens[p.current-1]
}

func (p *Parser) consume(kind lexer.TokenKind, message string) (lexer.Token, error) {
	if p.check(kind) {
		return p.advance(), nil
	}
	tokenName := p.tokenKindString(kind)
	current := p.peek()
	currentName := p.tokenKindString(current.Kind)
	return lexer.Token{}, p.errorf("Syntax Error", current.Pos,
		fmt.Sprintf("%s\n  Expected token: %s\n  Got: %s", message, tokenName, currentName))
}

func (p *Parser) consumeIdentifier(message string) (string, error) {
	if p.check(lexer.TokenIdentifier) {
		token := p.advance()
		if val, ok := token.Value.(string); ok {
			return val, nil
		}
	}
	current := p.peek()
	return "", p.errorf("Expected Identifier", current.Pos,
		fmt.Sprintf("%s\n  Got: %v at position", message, current.Value))
}

func (p *Parser) consumeString(message string) (string, error) {
	if p.check(lexer.TokenString) {
		token := p.advance()
		if val, ok := token.Value.(string); ok {
			return val, nil
		}
	}
	current := p.peek()
	return "", p.errorf("Expected String", current.Pos,
		fmt.Sprintf("%s\n  Got: %v", message, current.Value))
}

// tokenKindString provides human-readable token names for error messages
func (p *Parser) tokenKindString(kind lexer.TokenKind) string {
	names := map[lexer.TokenKind]string{
		lexer.TokenLParen:     "(\n    Use '(' to start a group or function call",
		lexer.TokenRParen:     ")\n    Use ')' to end a group or function call",
		lexer.TokenLBrace:     "{\n    Use '{' to start a code block",
		lexer.TokenRBrace:     "}\n    Use '}' to end a code block",
		lexer.TokenComma:      ",\n    Separate items with ','",
		lexer.TokenColon:      ":\n    Use ':' for type annotations",
		lexer.TokenEqual:      "=\n    Use '=' for assignment",
		lexer.TokenEqualEqual: "==\n    Use '==' for equality comparison",
		lexer.TokenSemicolon:  "; (semicolon is now optional)",
		lexer.TokenIdentifier: "identifier (variable or function name)",
		lexer.TokenString:     "string literal (\"...\")",
		lexer.TokenNumber:     "number",
	}
	if name, ok := names[kind]; ok {
		return name
	}
	return fmt.Sprintf("token type %d", kind)
}

// errorf formats parser errors in a modern compiler style
// Similar to Rust/Go compiler error messages with enhanced context
func (p *Parser) errorf(errorType string, pos lexer.Position, suggestion string) error {
	// Extract the line from source code for context
	lines := strings.Split(p.sourceCode, "\n")
	var contextLine string
	if pos.Line > 0 && pos.Line <= len(lines) {
		contextLine = lines[pos.Line-1]
	}

	// Create error pointer
	pointerLine := strings.Repeat(" ", pos.Column-1) + "^"

	// Format error with context
	errorMsg := fmt.Sprintf("\033[1;31m%s\033[0m at line %d, column %d\n",
		errorType, pos.Line, pos.Column)

	if contextLine != "" {
		errorMsg += fmt.Sprintf("  \033[1;33m→\033[0m %s\n", contextLine)
		errorMsg += fmt.Sprintf("    %s\n", pointerLine)
	}

	errorMsg += fmt.Sprintf("  \033[1;36mℹ\033[0m %s\n", suggestion)

	return fmt.Errorf(errorMsg)
}
