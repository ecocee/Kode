#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    /// Pointer to a reference-counted string.
    Str,
    Unit,
}

pub type LocalId = usize;
pub type BlockId = usize;

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Local(LocalId),
    Int(i64),
    Float(f64),
    Bool(bool),
    /// Index into `MirProgram::strings` (a static, never-freed string).
    Str(usize),
    NullStr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastKind {
    IntToFloat,
    FloatToInt,
    BoolToInt,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    Unary(UnOp, Operand),
    Binary(BinOp, Operand, Operand),
    Cast(CastKind, Operand),
}

/// Functions provided by the runtime library (`kode-rt`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RtFn {
    PrintInt,
    PrintFloat,
    PrintBool,
    PrintStr,
    PrintSep,
    PrintNl,
    StrConcat,
    StrEq,
    IntToStr,
    FloatToStr,
    BoolToStr,
    StrLen,
    AssertFail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callee {
    /// Index into `MirProgram::funcs`.
    User(usize),
    Rt(RtFn),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Inst {
    Assign {
        dst: LocalId,
        rv: Rvalue,
    },
    Call {
        dst: Option<LocalId>,
        callee: Callee,
        args: Vec<Operand>,
    },
    /// Add one owner to a string.
    Retain(Operand),
    /// Remove one owner from a string; frees it when the last owner leaves.
    Release(Operand),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Terminator {
    Goto(BlockId),
    Branch {
        cond: Operand,
        then_bb: BlockId,
        else_bb: BlockId,
    },
    Return(Option<Operand>),
    Unreachable,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub insts: Vec<Inst>,
    pub term: Terminator,
}

#[derive(Clone, Debug)]
pub struct MirFunc {
    pub name: String,
    /// The first `param_count` locals are the parameters.
    pub param_count: usize,
    pub ret: Ty,
    pub locals: Vec<LocalDecl>,
    /// Block 0 is the entry block.
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct MirProgram {
    pub funcs: Vec<MirFunc>,
    /// Static string literals.
    pub strings: Vec<String>,
    /// Index of the top-level script function.
    pub entry: usize,
}

impl MirFunc {
    pub fn param_tys(&self) -> Vec<Ty> {
        self.locals[..self.param_count].iter().map(|l| l.ty).collect()
    }

    pub fn operand_ty(&self, op: &Operand) -> Ty {
        match op {
            Operand::Local(id) => self.locals[*id].ty,
            Operand::Int(_) => Ty::Int,
            Operand::Float(_) => Ty::Float,
            Operand::Bool(_) => Ty::Bool,
            Operand::Str(_) | Operand::NullStr => Ty::Str,
        }
    }
}
