//! AST definitions used by PEG and interpreter

pub type Block<'a> = Vec<LineData<'a>>;

pub type DataBlock<'a> = Vec<DataDef<'a>>;

type E<'a> = Box<Expr<'a>>;

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(u32),
    Deref(E<'a>),
    DerefByte(E<'a>),
    Var(&'a str),
    VarAddress(&'a str),
    Invert(E<'a>),
    Add(E<'a>, E<'a>),
    Sub(E<'a>, E<'a>),
    Mul(E<'a>, E<'a>),
    Div(E<'a>, E<'a>),
    Mod(E<'a>, E<'a>),
    Lt(E<'a>, E<'a>),
    Gt(E<'a>, E<'a>),
    Leq(E<'a>, E<'a>),
    Geq(E<'a>, E<'a>),
    Eq(E<'a>, E<'a>),
    Neq(E<'a>, E<'a>),
    BitAnd(E<'a>, E<'a>),
    BitOr(E<'a>, E<'a>),
    BitXor(E<'a>, E<'a>),
    Shl(E<'a>, E<'a>),
    Shr(E<'a>, E<'a>),
    FunCall(&'a str, Vec<Expr<'a>>),
}

#[derive(Debug)]
pub struct LineData<'a> {
    /// The character index into the input string that this line starts on
    pub start_index: usize,
    pub line: Line<'a>,
}

impl<'a> From<(usize, Line<'a>)> for LineData<'a> {
    fn from((start_index, line): (usize, Line<'a>)) -> LineData {
        LineData { start_index, line }
    }
}

#[derive(Debug)]
pub enum Line<'a> {
    Assign(AssignTarget<'a>, Expr<'a>),
    For(&'a str, Expr<'a>, Expr<'a>, Block<'a>),
    While(Expr<'a>, Block<'a>),
    If(Expr<'a>, Block<'a>, Option<Block<'a>>),
    Goto(&'a str),
    Label(&'a str),
    // name, arg names, is_saveargs
    FunDeclaration(&'a str, Vec<&'a str>, Block<'a>, bool),
    // at the moment, the Line::Expr can only be a Expr::FunCall, otherwise
    // ambiguity arises (eg. 'end' getting parsed as Expr::Var("end") instead
    // of the end of a block)
    Expr(Expr<'a>),
}

/// The target of an assignment expression (ie. storing to a variable or an address)
#[derive(Debug)]
pub enum AssignTarget<'a> {
    Var(&'a str),
    Addr(Expr<'a>),
    ByteAddr(Expr<'a>),
}

#[derive(Debug)]
pub enum DataDef<'a> {
    Label(&'a str),
    Bytes(Vec<u8>),
}
