use std::collections::HashMap;

use crate::{
    ast::{
        BinaryOperator, Expression, ExpressionKind,
        Program, Statement, UnaryOperator,
    },
    diagnostics::KodeError,
    runtime::value::Value,
};

pub struct Interpreter {
    variables: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), KodeError> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }

        Ok(())
    }

    fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), KodeError> {
        match statement {
            Statement::Assign {
                name,
                value,
                line: _,
                column: _,
            } => {
                let result = self.evaluate(value)?;
                self.variables.insert(name.clone(), result);
            }

            Statement::Print {
                value,
                line: _,
                column: _,
            } => {
                let result = self.evaluate(value)?;
                println!("{}", result);
            }
        }

        Ok(())
    }

    fn evaluate(&self, expr: &Expression) -> Result<Value, KodeError> {
        match &expr.kind {
            ExpressionKind::Integer(value) => {
                Ok(Value::Integer(*value))
            }

            ExpressionKind::Float(value) => {
                Ok(Value::Float(*value))
            }

            ExpressionKind::String(value) => {
                Ok(Value::String(value.clone()))
            }

            ExpressionKind::Boolean(value) => {
                Ok(Value::Boolean(*value))
            }

            ExpressionKind::Variable(name) => {
                self.variables.get(name).cloned().ok_or_else(|| {
                    self.error(
                        expr,
                        &format!("undefined variable '{}'", name),
                    )
                })
            }

            ExpressionKind::Unary { operator, right } => {
                let value = self.evaluate(right)?;

                match (operator, value) {
                    (UnaryOperator::Negate, Value::Integer(v)) => {
                        v.checked_neg()
                            .map(Value::Integer)
                            .ok_or_else(|| self.error(
                                expr,
                                "integer overflow during negation",
                            ))
                    }

                    (UnaryOperator::Negate, Value::Float(v)) => {
                        Ok(Value::Float(-v))
                    }

                    (UnaryOperator::Negate, other) => Err(self.error(
                        expr,
                        &format!(
                            "cannot apply unary '-' to {}",
                            other.type_name()
                        ),
                    )),
                }
            }

            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let lhs = self.evaluate(left)?;
                let rhs = self.evaluate(right)?;

                self.evaluate_binary(expr, lhs, operator, rhs)
            }
        }
    }

    fn evaluate_binary(
        &self,
        expr: &Expression,
        left: Value,
        operator: &BinaryOperator,
        right: Value,
    ) -> Result<Value, KodeError> {
        use BinaryOperator::*;
        use Value::*;

        // String concatenation.
        if let (String(a), String(b), Add) = (&left, &right, operator) {
            return Ok(String(format!("{}{}", a, b)));
        }

        // Equality works for matching value types.
        if matches!(operator, Equal | NotEqual) {
            let equal = match (&left, &right) {
                (Integer(a), Integer(b)) => a == b,
                (Float(a), Float(b)) => a == b,
                (Integer(a), Float(b)) => (*a as f64) == *b,
                (Float(a), Integer(b)) => *a == (*b as f64),
                (String(a), String(b)) => a == b,
                (Boolean(a), Boolean(b)) => a == b,
                _ => false,
            };

            return Ok(Boolean(if matches!(operator, Equal) {
                equal
            } else {
                !equal
            }));
        }

        // Numeric operations.
        match (&left, &right) {
            (Integer(a), Integer(b)) => {
                match operator {
                    Add => a.checked_add(*b).map(Integer),
                    Subtract => a.checked_sub(*b).map(Integer),
                    Multiply => a.checked_mul(*b).map(Integer),

                    // Division always returns a float.
                    Divide => {
                        if *b == 0 {
                            return Err(self.error(expr, "division by zero"));
                        }

                        Some(Float(*a as f64 / *b as f64))
                    }

                    Less => return Ok(Boolean(a < b)),
                    LessEqual => return Ok(Boolean(a <= b)),
                    Greater => return Ok(Boolean(a > b)),
                    GreaterEqual => return Ok(Boolean(a >= b)),

                    _ => None,
                }
                .ok_or_else(|| self.error(expr, "integer overflow"))
            }

            _ => {
                let a = left.as_number();
                let b = right.as_number();

                if let (Some(a), Some(b)) = (a, b) {
                    if matches!(operator, Divide) && b == 0.0 {
                        return Err(self.error(expr, "division by zero"));
                    }

                    let result = match operator {
                        Add => Float(a + b),
                        Subtract => Float(a - b),
                        Multiply => Float(a * b),
                        Divide => Float(a / b),

                        Less => return Ok(Boolean(a < b)),
                        LessEqual => return Ok(Boolean(a <= b)),
                        Greater => return Ok(Boolean(a > b)),
                        GreaterEqual => return Ok(Boolean(a >= b)),

                        _ => {
                            return Err(self.error(
                                expr,
                                "unsupported numeric operation",
                            ));
                        }
                    };

                    if let Float(value) = result {
                        if !value.is_finite() {
                            return Err(self.error(
                                expr,
                                "floating-point result is not finite",
                            ));
                        }

                        return Ok(Float(value));
                    }
                }

                Err(self.error(
                    expr,
                    &format!(
                        "unsupported operation between {} and {}",
                        left.type_name(),
                        right.type_name()
                    ),
                ))
            }
        }
    }

    fn error(&self, expr: &Expression, message: &str) -> KodeError {
        KodeError::new(
            "Runtime",
            message,
            expr.line,
            expr.column,
        )
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}