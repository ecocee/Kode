package parser

import (
	"fmt"
	"path/filepath"
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

	filePrefix := strings.TrimSuffix(filepath.Base(filePath), filepath.Ext(filePath))
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
	if p.match(lexer.TokenLet) {
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

	stmt := ast.LetStmt{Name: name, Type: typ, Value: value}
	return stmt, nil
}

func (p *Parser) functionDefinition() (ast.Statement, error) {
	isMain := p.match(lexer.TokenMain)

	var name string
	if !isMain {
		n, err := p.consumeIdentifier("Expected function name after 'func'")
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
	if !p.check(lexer.TokenLBrace) && !p.check(lexer.TokenEqual) {
		rt, err := p.parseType()
		if err != nil {
			return nil, err
		}
		returnType = rt
	}

	var body interface{}
	var isExprBody bool
	if p.match(lexer.TokenEqual) {
		// Expression body
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
		return nil, fmt.Errorf("Expected '=' or '{' after function signature")
	}

	return ast.FunctionDefStmt{
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

	if _, err := p.consume(lexer.TokenColon, "Expected ':' after parameter name"); err != nil {
		return ast.Param{}, err
	}

	typ, err := p.parseType()
	if err != nil {
		return ast.Param{}, err
	}

	return ast.Param{Name: name, Type: typ}, nil
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

	if _, err := p.consume(lexer.TokenSemicolon, "Expected ';' after constant declaration"); err != nil {
		return nil, err
	}

	return ast.ConstDeclStmt{Name: name, Type: typ, Value: value}, nil
}

func (p *Parser) structDeclaration() (ast.Statement, error) {
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

	return ast.StructDeclStmt{Name: name, Fields: fields}, nil
}

func (p *Parser) enumDeclaration() (ast.Statement, error) {
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

	return ast.EnumDeclStmt{Name: name, Variants: variants}, nil
}

func (p *Parser) traitDeclaration() (ast.Statement, error) {
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

	return ast.TraitDeclStmt{Name: name, Signatures: signatures}, nil
}

func (p *Parser) implDeclaration() (ast.Statement, error) {
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

	return ast.ImplDeclStmt{Trait: trait, Type: typ, Methods: methods}, nil
}

func (p *Parser) serviceDeclaration() (ast.Statement, error) {
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

	return ast.ServiceDeclStmt{Name: name, Routes: routes}, nil
}

func (p *Parser) moduleDeclaration() (ast.Statement, error) {
	name, err := p.consumeIdentifier("Expected module name")
	if err != nil {
		return nil, err
	}

	return ast.ModuleDeclStmt{Name: name}, nil
}

func (p *Parser) importDeclaration() (ast.Statement, error) {
	name, err := p.consumeIdentifier("Expected module name after import/use")
	if err != nil {
		return nil, err
	}

	return ast.ImportStmt{Path: name}, nil
}

func (p *Parser) spawnStatement() (ast.Statement, error) {
	call, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenSemicolon, "Expected ';' after spawn"); err != nil {
		return nil, err
	}

	return ast.SpawnStmt{Call: call}, nil
}

func (p *Parser) statement() (ast.Statement, error) {
	if p.match(lexer.TokenLBrace) {
		stmts, err := p.block()
		if err != nil {
			return nil, err
		}
		return ast.BlockStmt{Statements: stmts}, nil
	} else if p.match(lexer.TokenIf) {
		return p.ifStatement()
	} else if p.match(lexer.TokenWhile) {
		return p.whileStatement()
	} else if p.match(lexer.TokenFor) {
		return p.forStatement()
	} else if p.match(lexer.TokenReturn) {
		return p.returnStatement()
	} else if p.match(lexer.TokenBreak) {
		return ast.BreakStmt{}, nil
	} else if p.match(lexer.TokenContinue) {
		return ast.ContinueStmt{}, nil
	} else if p.match(lexer.TokenDefer) {
		return p.deferStatement()
	} else if p.match(lexer.TokenMatch) {
		return p.matchStatement()
	} else if p.check(lexer.TokenIdentifier) && p.current+1 < len(p.tokens) && p.tokens[p.current+1].Kind == lexer.TokenColon {
		// Handle name : type = expression
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
			Name:  name,
			Type:  typ,
			Value: value,
		}, nil
	} else if p.check(lexer.TokenIdentifier) && p.current+1 < len(p.tokens) && p.tokens[p.current+1].Kind == lexer.TokenWalrus {
		// Handle name := expression
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
			Name:  name,
			Type:  nil, // Inferred
			Value: value,
		}
		return stmt, nil
	}
	return p.expressionStatement()
}

func (p *Parser) returnStatement() (ast.Statement, error) {
	var value ast.Expression
	if !p.isAtEnd() && !p.check(lexer.TokenRBrace) && !p.check(lexer.TokenSemicolon) {
		v, err := p.expression()
		if err != nil {
			return nil, err
		}
		value = v
	}

	return ast.ReturnStmt{Value: value}, nil
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
		Condition:  condition,
		ThenBranch: thenBranch,
		ElseBranch: elseBranch,
	}, nil
}

func (p *Parser) whileStatement() (ast.Statement, error) {
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

	return ast.WhileStmt{Condition: condition, Body: body}, nil
}

func (p *Parser) forStatement() (ast.Statement, error) {
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
	if _, err := p.consume(lexer.TokenSemicolon, "Expected ';' after init"); err != nil {
		return nil, err
	}

	var cond ast.Expression
	if !p.check(lexer.TokenSemicolon) {
		c, err := p.expression()
		if err != nil {
			return nil, err
		}
		cond = c
	}
	if _, err := p.consume(lexer.TokenSemicolon, "Expected ';' after condition"); err != nil {
		return nil, err
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
		Init:      init,
		Condition: cond,
		Incr:      incr,
		Body:      body,
	}, nil
}

func (p *Parser) deferStatement() (ast.Statement, error) {
	call, err := p.expression()
	if err != nil {
		return nil, err
	}

	if _, err := p.consume(lexer.TokenSemicolon, "Expected ';' after defer"); err != nil {
		return nil, err
	}

	return ast.DeferStmt{Call: call}, nil
}

func (p *Parser) matchStatement() (ast.Statement, error) {
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

		if !p.match(lexer.TokenComma) {
			break
		}
	}

	if _, err := p.consume(lexer.TokenRBrace, "Expected '}' after match"); err != nil {
		return nil, err
	}

	return ast.MatchStmt{Expr: expr, Cases: cases}, nil
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
	expr, err := p.expression()
	if err != nil {
		return nil, err
	}

	// if the expression is a binary assign with an identifier on the left, treat
	// it as an assignment statement rather than a comparison (assignment
	// operator is parsed as OpAssign).
	if bin, ok := expr.(ast.BinaryExpr); ok && bin.Op == ast.OpAssign {
		if id, ok := bin.Left.(ast.IdentifierExpr); ok {
			return ast.AssignStmt{Name: id.Name, Value: bin.Right}, nil
		}
	}

	return ast.ExprStmt{Expr: expr}, nil
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

	return expr, nil
}

func (p *Parser) logicOr() (ast.Expression, error) {
	expr, err := p.logicAnd()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenOr) {
		right, err := p.logicAnd()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: ast.OpOr, Right: right}
	}

	return expr, nil
}

func (p *Parser) logicAnd() (ast.Expression, error) {
	expr, err := p.bitwiseOr()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenAnd) {
		right, err := p.bitwiseOr()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: ast.OpAnd, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseOr() (ast.Expression, error) {
	expr, err := p.bitwiseXor()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitOr) {
		right, err := p.bitwiseXor()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: ast.OpBitOr, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseXor() (ast.Expression, error) {
	expr, err := p.bitwiseAnd()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitXor) {
		right, err := p.bitwiseAnd()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: ast.OpBitXor, Right: right}
	}

	return expr, nil
}

func (p *Parser) bitwiseAnd() (ast.Expression, error) {
	expr, err := p.equality()
	if err != nil {
		return nil, err
	}

	for p.match(lexer.TokenBitAnd) {
		right, err := p.equality()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: ast.OpBitAnd, Right: right}
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
		right, err := p.comparison()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: op, Right: right}
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
		right, err := p.shift()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: op, Right: right}
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
		right, err := p.term()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: op, Right: right}
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
		right, err := p.factor()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: op, Right: right}
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
		right, err := p.unary()
		if err != nil {
			return nil, err
		}
		expr = ast.BinaryExpr{Left: expr, Op: op, Right: right}
	}

	return expr, nil
}

func (p *Parser) unary() (ast.Expression, error) {
	if p.match(lexer.TokenMinus, lexer.TokenNot, lexer.TokenBitNot) {
		op := ast.OpNegate
		if p.previous().Kind == lexer.TokenNot {
			op = ast.OpNot
		}
		right, err := p.unary()
		if err != nil {
			return nil, err
		}
		return ast.UnaryExpr{Op: op, Expr: right}, nil
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
			expr = ast.ArrayAccessExpr{Array: expr, Index: index}
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

	return ast.CallExpr{Callee: callee, Arguments: arguments}, nil
}

func (p *Parser) primary() (ast.Expression, error) {
	var expr ast.Expression
	if p.match(lexer.TokenNumber) {
		if val, ok := p.previous().Value.(int64); ok {
			expr = ast.NumberExpr{Value: val}
		}
	} else if p.match(lexer.TokenFloat) {
		if val, ok := p.previous().Value.(float64); ok {
			expr = ast.FloatExpr{Value: val}
		}
	} else if p.match(lexer.TokenBool) {
		if val, ok := p.previous().Value.(bool); ok {
			expr = ast.BoolExpr{Value: val}
		}
	} else if p.match(lexer.TokenString) {
		if val, ok := p.previous().Value.(string); ok {
			expr = ast.StringExpr{Value: val}
		}
	} else if p.match(lexer.TokenIdentifier) {
		if val, ok := p.previous().Value.(string); ok {
			expr = ast.IdentifierExpr{Name: val}
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
	} else if p.match(lexer.TokenChan) {
		typ, err := p.parseType()
		if err != nil {
			return nil, err
		}
		expr = ast.ChanExpr{Type: typ}
	} else {
		return nil, fmt.Errorf("Expected expression")
	}

	if p.match(lexer.TokenPlusPlus) {
		expr = ast.UnaryExpr{Op: ast.OpPostInc, Expr: expr}
	} else if p.match(lexer.TokenMinusMinus) {
		expr = ast.UnaryExpr{Op: ast.OpPostDec, Expr: expr}
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

	return ast.ArrayExpr{Elements: elements}, nil
}

func (p *Parser) closure() (ast.Expression, error) {
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

	return ast.ClosureExpr{Params: params, Body: body}, nil
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
	return lexer.Token{}, fmt.Errorf("%s at %v", message, p.peek().Pos)
}

func (p *Parser) consumeIdentifier(message string) (string, error) {
	if p.check(lexer.TokenIdentifier) {
		token := p.advance()
		if val, ok := token.Value.(string); ok {
			return val, nil
		}
	}
	return "", fmt.Errorf("%s at %v", message, p.peek().Pos)
}

func (p *Parser) consumeString(message string) (string, error) {
	if p.check(lexer.TokenString) {
		token := p.advance()
		if val, ok := token.Value.(string); ok {
			return val, nil
		}
	}
	return "", fmt.Errorf("%s at %v", message, p.peek().Pos)
}
