#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    /// Pointer to a reference-counted heap object (string, list, map, class instance).
    Ref,
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
    /// A null reference (`none`, or a variable that has no value yet).
    NullRef,
    /// The address of generated task thunk `i` (see `MirProgram::thunks`).
    Thunk(usize),
    /// The address of user function `i`.
    FuncAddr(usize),
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
    /// Internal only: used by optimizations such as `x % 2 == 0`.
    BitAnd,
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
    IntToBool,
    FloatToBits,
    BitsToFloat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    Unary(UnOp, Operand),
    Binary(BinOp, Operand, Operand),
    Cast(CastKind, Operand),
    /// `cond ? a : b` without a branch.
    Select(Operand, Operand, Operand),
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
    ListNew,
    ListPush,
    ListGet,
    ListSet,
    ListLen,
    /// Lists of `bool` use one byte per item.
    ListNewPacked,
    ListPush8,
    ListGet8,
    ListSet8,
    MapNew,
    MapSet,
    MapGet,
    MapHas,
    MapLen,
    MapKeyAt,
    MapValAt,
    MapKeys,
    ObjNew,
    ErrThrow,
    ErrPending,
    ErrTake,
    Uncaught,
    UnwrapFail,
    Exit,
    ListOob,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callee {
    /// Index into `MirProgram::funcs`.
    User(usize),
    Rt(RtFn),
    /// Index into `MirProgram::std_sigs`: a runtime library function called by name.
    Std(usize),
}

/// A runtime library function used by the program.
#[derive(Clone, Debug)]
pub struct StdSig {
    pub name: &'static str,
    pub params: Vec<Ty>,
    pub ret: Ty,
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
    /// Read a field: `dst = *(base + offset)`.
    Load {
        dst: LocalId,
        base: Operand,
        offset: i32,
        ty: Ty,
    },
    /// Write a field: `*(base + offset) = value`.
    Store {
        base: Operand,
        offset: i32,
        value: Operand,
    },
    /// Add one owner to a heap object.
    Retain(Operand),
    /// Remove one owner; frees the object when the last owner leaves.
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
    /// Small generated functions that start tasks (see `spawn`).
    pub thunks: Vec<MirFunc>,
    pub std_sigs: Vec<StdSig>,
}

impl MirFunc {
    pub fn param_tys(&self) -> Vec<Ty> {
        self.locals[..self.param_count].iter().map(|l| l.ty).collect()
    }

    pub fn operand_ty(&self, op: &Operand) -> Ty {
        match op {
            Operand::Local(id) => self.locals[*id].ty,
            Operand::Int(_) | Operand::Thunk(_) | Operand::FuncAddr(_) => Ty::Int,
            Operand::Float(_) => Ty::Float,
            Operand::Bool(_) => Ty::Bool,
            Operand::Str(_) | Operand::NullRef => Ty::Ref,
        }
    }
}