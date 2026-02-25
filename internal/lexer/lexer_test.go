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
