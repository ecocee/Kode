package lexer

import (
	"testing"
)

func TestLexer(t *testing.T) {
	tests := []struct {
		input    string
		expected []TokenKind
	}{
		{
			input:    "x := 42",
			expected: []TokenKind{TokenIdentifier, TokenWalrus, TokenNumber, TokenEOF},
		},
		{
			input:    "func add(a: int, b: int) int { return a + b }",
			expected: []TokenKind{TokenFunc, TokenIdentifier, TokenLParen, TokenIdentifier, TokenColon, TokenInt, TokenComma, TokenIdentifier, TokenColon, TokenInt, TokenRParen, TokenInt, TokenLBrace, TokenReturn, TokenIdentifier, TokenPlus, TokenIdentifier, TokenRBrace, TokenEOF},
		},
		{
			input: `// comment
x = 1 /* block */`,
			expected: []TokenKind{TokenIdentifier, TokenEqual, TokenNumber, TokenEOF},
		},
		{
			input:    `"hello" + "world"`,
			expected: []TokenKind{TokenString, TokenPlus, TokenString, TokenEOF},
		},
		{
			input:    `a && b || !c`,
			expected: []TokenKind{TokenIdentifier, TokenAnd, TokenIdentifier, TokenOr, TokenNot, TokenIdentifier, TokenEOF},
		},
		{
			input:    "3.14 <= 2.7",
			expected: []TokenKind{TokenFloat, TokenLessThanOrEqual, TokenFloat, TokenEOF},
		},
		// regression test for minus lexing
		{
			input:    "a - b",
			expected: []TokenKind{TokenIdentifier, TokenMinus, TokenIdentifier, TokenEOF},
		},
		// verify arrow tokens from both flavors
		{
			input:    "x => y",
			expected: []TokenKind{TokenIdentifier, TokenArrow, TokenIdentifier, TokenEOF},
		},
		{
			input:    "x -> y",
			expected: []TokenKind{TokenIdentifier, TokenArrow, TokenIdentifier, TokenEOF},
		},
	}

	for _, test := range tests {
		lexer := NewLexer(test.input)
		tokens, err := lexer.Tokenize()
		if err != nil {
			t.Errorf("Lexer error: %v", err)
			continue
		}

		if len(tokens) != len(test.expected) {
			t.Errorf("Expected %d tokens, got %d", len(test.expected), len(tokens))
			continue
		}

		for i, token := range tokens {
			if token.Kind != test.expected[i] {
				t.Errorf("Token %d: expected %v, got %v", i, test.expected[i], token.Kind)
			}
		}
	}
}
