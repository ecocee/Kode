package lexer

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

type TokenKind int

const (
	// Literals
	TokenNumber TokenKind = iota
	TokenFloat
	TokenBool
	TokenString
	TokenIdentifier

	// Keywords
	TokenLet
	TokenFunc
	TokenReturn
	TokenIf
	TokenElse
	TokenFor
	TokenWhile
	TokenBreak
	TokenContinue
	TokenMatch
	TokenStruct
	TokenEnum
	TokenImpl
	TokenTrait
	TokenUse
	TokenMod
	TokenPub
	TokenExport
	TokenAs
	TokenConst
	TokenAsync
	TokenAwait
	TokenSpawn
	TokenSelect
	TokenChannel
	TokenImport
	TokenService
	TokenDefer

	// Entry
	TokenEntry

	// Types
	TokenInt
	TokenStringType
	TokenBoolType
	TokenFloatType

	// Additional keywords
	TokenGo
	TokenMain
	TokenIn
	TokenFrom
	TokenCatch
	TokenUnderscore
	TokenTrue
	TokenFalse

	// Channel
	TokenChan

	// HTTP
	TokenHttp
	TokenRoute

	// Operators
	TokenPlus
	TokenMinus
	TokenStar
	TokenSlash
	TokenPercent
	TokenPlusPlus
	TokenMinusMinus
	TokenEqual
	TokenEqualEqual
	TokenNotEqual
	TokenLessThan
	TokenGreaterThan
	TokenLessThanOrEqual
	TokenGreaterThanOrEqual
	TokenAnd
	TokenOr
	TokenNot
	TokenBitAnd
	TokenBitOr
	TokenBitXor
	TokenBitNot
	TokenBitShl
	TokenBitShr

	// Walrus operator
	TokenWalrus

	// Symbols
	TokenLParen
	TokenRParen
	TokenLBrace
	TokenRBrace
	TokenLBracket
	TokenRBracket
	TokenComma
	TokenSemicolon
	TokenColon
	TokenArrow
	TokenDot

	TokenEOF
)

type Position struct {
	Line   int
	Column int
}

type Token struct {
	Kind  TokenKind
	Value interface{}
	Pos   Position
}

type Lexer struct {
	input  string
	pos    int
	line   int
	column int
}

var keywords = map[string]TokenKind{
	"let":      TokenLet,
	"func":     TokenFunc,
	"return":   TokenReturn,
	"if":       TokenIf,
	"else":     TokenElse,
	"for":      TokenFor,
	"while":    TokenWhile,
	"break":    TokenBreak,
	"continue": TokenContinue,
	"match":    TokenMatch,
	"struct":   TokenStruct,
	"enum":     TokenEnum,
	"impl":     TokenImpl,
	"trait":    TokenTrait,
	"use":      TokenUse,
	"mod":      TokenMod,
	"pub":      TokenPub,
	"export":   TokenExport,
	"as":       TokenAs,
	"const":    TokenConst,
	"async":    TokenAsync,
	"await":    TokenAwait,
	"spawn":    TokenSpawn,
	"select":   TokenSelect,
	"chan":     TokenChan,
	"defer":    TokenDefer,
	"import":   TokenImport,
	"from":     TokenFrom,
	"service":  TokenService,
	"entry":    TokenEntry,
	"int":      TokenInt,
	"string":   TokenStringType,
	"bool":     TokenBoolType,
	"float":    TokenFloatType,
	"true":     TokenBool,
	"false":    TokenBool,
}

func NewLexer(input string) *Lexer {
	return &Lexer{
		input:  input,
		pos:    0,
		line:   1,
		column: 1,
	}
}

func (l *Lexer) GetPosition() Position {
	return Position{Line: l.line, Column: l.column}
}

func (l *Lexer) Tokenize() ([]Token, error) {
	var tokens []Token

	// Skip UTF-8 BOM if present
	if len(l.input) >= 3 && l.input[0] == '\xef' && l.input[1] == '\xbb' && l.input[2] == '\xbf' {
		l.pos = 3
		l.column = 1 // Reset column since we skipped
	}

	for l.peekChar() != 0 {
		ch := l.peekChar()
		switch {
		case unicode.IsSpace(ch):
			if ch == '\n' {
				l.line++
				l.column = 1
			} else {
				l.column++
			}
			l.consumeChar()
		case ch == '/' && l.peekCharAt(1) == '/':
			// Line comment
			for l.peekChar() != 0 && l.peekChar() != '\n' {
				l.consumeChar()
			}
		case ch == '/' && l.peekCharAt(1) == '*':
			// Block comment
			if err := l.skipBlockComment(); err != nil {
				return nil, err
			}
		case ch == '+':
			if l.peekCharAt(1) == '+' {
				tokens = append(tokens, Token{Kind: TokenPlusPlus, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenPlus, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '-':
			if l.peekCharAt(1) == '-' {
				tokens = append(tokens, Token{Kind: TokenMinusMinus, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else if l.peekCharAt(1) == '>' {
				tokens = append(tokens, Token{Kind: TokenArrow, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenMinus, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '*':
			tokens = append(tokens, Token{Kind: TokenStar, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '/':
			tokens = append(tokens, Token{Kind: TokenSlash, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '%':
			tokens = append(tokens, Token{Kind: TokenPercent, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '.':
			tokens = append(tokens, Token{Kind: TokenDot, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '=':
			if l.peekCharAt(1) == '=' {
				tokens = append(tokens, Token{Kind: TokenEqualEqual, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else if l.peekCharAt(1) == ':' {
				// Note: := is : followed by =, but we check = followed by :
				// Actually, since we read left to right, when we see :, we should check if next is =
				// But here we're at =, so if previous was :, but that's not how it works.
				// Actually, := should be handled when we see :
				// Let me change this.
				// Better to handle := when we see :
				// Let me find the : case.
			} else {
				tokens = append(tokens, Token{Kind: TokenEqual, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '!':
			if l.peekCharAt(1) == '=' {
				tokens = append(tokens, Token{Kind: TokenNotEqual, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenNot, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '<':
			if l.peekCharAt(1) == '=' {
				tokens = append(tokens, Token{Kind: TokenLessThanOrEqual, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else if l.peekCharAt(1) == '<' {
				tokens = append(tokens, Token{Kind: TokenBitShl, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenLessThan, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '>':
			if l.peekCharAt(1) == '=' {
				tokens = append(tokens, Token{Kind: TokenGreaterThanOrEqual, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else if l.peekCharAt(1) == '>' {
				tokens = append(tokens, Token{Kind: TokenBitShr, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenGreaterThan, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '&':
			if l.peekCharAt(1) == '&' {
				tokens = append(tokens, Token{Kind: TokenAnd, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenBitAnd, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '|':
			if l.peekCharAt(1) == '|' {
				tokens = append(tokens, Token{Kind: TokenOr, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenBitOr, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '^':
			tokens = append(tokens, Token{Kind: TokenBitXor, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '~':
			tokens = append(tokens, Token{Kind: TokenBitNot, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '(':
			tokens = append(tokens, Token{Kind: TokenLParen, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == ')':
			tokens = append(tokens, Token{Kind: TokenRParen, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '{':
			tokens = append(tokens, Token{Kind: TokenLBrace, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '}':
			tokens = append(tokens, Token{Kind: TokenRBrace, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == '[':
			tokens = append(tokens, Token{Kind: TokenLBracket, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == ']':
			tokens = append(tokens, Token{Kind: TokenRBracket, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == ',':
			tokens = append(tokens, Token{Kind: TokenComma, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == ';':
			tokens = append(tokens, Token{Kind: TokenSemicolon, Pos: l.GetPosition()})
			l.consumeChar()
			l.column++
		case ch == ':':
			if l.peekCharAt(1) == '=' {
				tokens = append(tokens, Token{Kind: TokenWalrus, Pos: l.GetPosition()})
				l.consumeChar()
				l.consumeChar()
				l.column += 2
			} else {
				tokens = append(tokens, Token{Kind: TokenColon, Pos: l.GetPosition()})
				l.consumeChar()
				l.column++
			}
		case ch == '"':
			token, err := l.readString()
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
		case unicode.IsDigit(ch):
			token := l.readNumber()
			tokens = append(tokens, token)
		case unicode.IsLetter(ch) || ch == '_':
			token := l.readIdentifierOrKeyword()
			tokens = append(tokens, token)
		default:
			return nil, fmt.Errorf("unexpected character '%c' at %v", ch, l.GetPosition())
		}
	}

	tokens = append(tokens, Token{Kind: TokenEOF, Pos: l.GetPosition()})
	return tokens, nil
}

func (l *Lexer) skipBlockComment() error {
	l.consumeChar() // /
	l.consumeChar() // *
	nesting := 1
	for nesting > 0 && l.peekChar() != 0 {
		if l.peekChar() == '*' && l.peekCharAt(1) == '/' {
			l.consumeChar()
			l.consumeChar()
			nesting--
		} else if l.peekChar() == '/' && l.peekCharAt(1) == '*' {
			l.consumeChar()
			l.consumeChar()
			nesting++
		} else {
			if l.peekChar() == '\n' {
				l.line++
				l.column = 1
			} else {
				l.column++
			}
			l.consumeChar()
		}
	}
	if nesting > 0 {
		return fmt.Errorf("unterminated block comment")
	}
	return nil
}

func (l *Lexer) readNumber() Token {
	start := l.pos
	startCol := l.column
	isFloat := false

	for unicode.IsDigit(l.peekChar()) || (l.peekChar() == '.' && !isFloat) {
		if l.peekChar() == '.' {
			if unicode.IsDigit(l.peekCharAt(1)) {
				isFloat = true
			} else {
				break
			}
		}
		l.consumeChar()
		l.column++
	}

	numStr := l.input[start:l.pos]

	if isFloat {
		val, _ := strconv.ParseFloat(numStr, 64)
		return Token{Kind: TokenFloat, Value: val, Pos: Position{Line: l.line, Column: startCol}}
	} else {
		val, _ := strconv.ParseInt(numStr, 10, 64)
		return Token{Kind: TokenNumber, Value: val, Pos: Position{Line: l.line, Column: startCol}}
	}
}

func (l *Lexer) readString() (Token, error) {
	startLine := l.line
	startCol := l.column
	l.consumeChar() // "
	l.column++
	var content strings.Builder
	escaped := false

	for l.peekChar() != 0 {
		ch := l.peekChar()
		l.consumeChar()
		l.column++

		if escaped {
			switch ch {
			case 'n':
				content.WriteByte('\n')
			case 'r':
				content.WriteByte('\r')
			case 't':
				content.WriteByte('\t')
			case '\\':
				content.WriteByte('\\')
			case '"':
				content.WriteByte('"')
			default:
				return Token{}, fmt.Errorf("invalid escape sequence \\%c at line %d, column %d", ch, l.line, l.column)
			}
			escaped = false
		} else if ch == '\\' {
			escaped = true
		} else if ch == '"' {
			return Token{Kind: TokenString, Value: content.String(), Pos: Position{Line: l.line, Column: l.column - len(content.String()) - 2}}, nil
		} else {
			if ch == '\n' {
				l.line++
				l.column = 1
			}
			content.WriteRune(ch)
		}
	}

	return Token{}, fmt.Errorf("unterminated string at line %d, column %d", startLine, startCol)
}

func (l *Lexer) readIdentifierOrKeyword() Token {
	start := l.pos
	startCol := l.column

	for unicode.IsLetter(l.peekChar()) || unicode.IsDigit(l.peekChar()) || l.peekChar() == '_' {
		l.consumeChar()
		l.column++
	}

	ident := l.input[start:l.pos]

	kind, ok := keywords[ident]
	if !ok {
		kind = TokenIdentifier
	}

	var value interface{}
	switch kind {
	case TokenBool:
		value = ident == "true"
	case TokenIdentifier:
		value = ident
	}

	return Token{Kind: kind, Value: value, Pos: Position{Line: l.line, Column: startCol}}
}

func (l *Lexer) peekChar() rune {
	if l.pos >= len(l.input) {
		return 0
	}
	return rune(l.input[l.pos])
}

func (l *Lexer) peekCharAt(offset int) rune {
	idx := l.pos + offset
	if idx >= len(l.input) {
		return 0
	}
	return rune(l.input[idx])
}

func (l *Lexer) consumeChar() {
	if l.pos < len(l.input) {
		if l.peekChar() == '\n' {
			l.line++
			l.column = 1
		} else {
			l.column++
		}
		l.pos++
	}
}
