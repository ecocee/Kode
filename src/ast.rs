#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        name: String,
        value: Expression,
        line: usize,
        column: usize,
    },

    Print {
        value: Expression,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    Variable(String),

    Unary {
        operator: UnaryOperator,
        right: Box<Expression>,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}