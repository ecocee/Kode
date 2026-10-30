use crate::{
    ast::{
        BinaryOperator, Expression, ExpressionKind, Program,
        Statement, UnaryOperator,
    },
    diagnostics::KodeError,
    lexer::{Token, TokenKind},
};

pub fn parse(tokens: Vec<Token>) -> Result<Program, KodeError> {
    Parser { tokens, current: 0 }.program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn program(&mut self) -> Result<Program, KodeError> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            statements.push(self.statement()?);

            if !self.is_at_end()
                && !matches!(self.peek().kind, TokenKind::Newline)
            {
                return Err(self.error("expected a new line after statement"));
            }

            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Statement, KodeError> {
        if matches!(self.peek().kind, TokenKind::Print) {
            return self.print_statement();
        }

        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            if matches!(
                self.tokens.get(self.current + 1).map(|t| &t.kind),
                Some(TokenKind::Equal)
            ) {
                let token = self.advance().clone();
                self.advance(); // '='

                let value = self.expression()?;

                return Ok(Statement::Assign {
                    name,
                    value,
                    line: token.line,
                    column: token.column,
                });
            }
        }

        Err(self.error(
            "expected a variable assignment or print(...) statement",
        ))
    }

    fn print_statement(&mut self) -> Result<Statement, KodeError> {
        let token = self.advance().clone();

        self.consume(
            |kind| matches!(kind, TokenKind::LeftParen),
            "expected '(' after print",
        )?;

        let value = self.expression()?;

        self.consume(
            |kind| matches!(kind, TokenKind::RightParen),
            "expected ')' after print expression",
        )?;

        Ok(Statement::Print {
            value,
            line: token.line,
            column: token.column,
        })
    }

    fn expression(&mut self) -> Result<Expression, KodeError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, KodeError> {
        let mut expr = self.comparison()?;

        loop {
            let operator = match self.peek().kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => break,
            };

            let token = self.advance().clone();
            let right = self.comparison()?;

            expr = self.binary(expr, operator, right, &token);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, KodeError> {
        let mut expr = self.term()?;

        loop {
            let operator = match self.peek().kind {
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => break,
            };

            let token = self.advance().clone();
            let right = self.term()?;

            expr = self.binary(expr, operator, right, &token);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, KodeError> {
        let mut expr = self.factor()?;

        loop {
            let operator = match self.peek().kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            let token = self.advance().clone();
            let right = self.factor()?;

            expr = self.binary(expr, operator, right, &token);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, KodeError> {
        let mut expr = self.unary()?;

        loop {
            let operator = match self.peek().kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                _ => break,
            };

            let token = self.advance().clone();
            let right = self.unary()?;

            expr = self.binary(expr, operator, right, &token);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, KodeError> {
        if matches!(self.peek().kind, TokenKind::Minus) {
            let token = self.advance().clone();
            let right = self.unary()?;

            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::Negate,
                    right: Box::new(right),
                },
                line: token.line,
                column: token.column,
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, KodeError> {
        let token = self.advance().clone();

        let kind = match token.kind {
            TokenKind::Integer(value) => ExpressionKind::Integer(value),
            TokenKind::Float(value) => ExpressionKind::Float(value),
            TokenKind::String(value) => ExpressionKind::String(value),
            TokenKind::True => ExpressionKind::Boolean(true),
            TokenKind::False => ExpressionKind::Boolean(false),

            TokenKind::Identifier(name) => {
                ExpressionKind::Variable(name)
            }

            TokenKind::LeftParen => {
                let expr = self.expression()?;

                self.consume(
                    |kind| matches!(kind, TokenKind::RightParen),
                    "expected ')' after expression",
                )?;

                return Ok(expr);
            }

            _ => {
                return Err(KodeError::new(
                    "Parser",
                    "expected a value or expression",
                    token.line,
                    token.column,
                ));
            }
        };

        Ok(Expression {
            kind,
            line: token.line,
            column: token.column,
        })
    }

    fn binary(
        &self,
        left: Expression,
        operator: BinaryOperator,
        right: Expression,
        token: &Token,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
            line: token.line,
            column: token.column,
        }
    }

    fn consume(
        &mut self,
        condition: impl FnOnce(&TokenKind) -> bool,
        message: &str,
    ) -> Result<(), KodeError> {
        if condition(&self.peek().kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        &self.tokens[self.current - 1]
    }

    fn error(&self, message: &str) -> KodeError {
        let token = self.peek();

        KodeError::new(
            "Parser",
            message,
            token.line,
            token.column,
        )
    }
}