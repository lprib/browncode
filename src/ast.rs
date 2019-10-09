pub type Block<'a> = Vec<Line<'a>>;

pub type DataBlock<'a> = Vec<DataDef<'a>>;

type E<'a> = Box<Expr<'a>>;

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(u32),
    Deref(E<'a>),
    Var(&'a str),
    VarAddress(&'a str),
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
    FunCall(&'a str, Vec<Expr<'a>>)
}

#[derive(Debug)]
pub enum Line<'a> {
    Assign(&'a str, Expr<'a>),
    For(&'a str, Expr<'a>, Expr<'a>, Block<'a>),
    While(Expr<'a>, Block<'a>),
    If(Expr<'a>, Block<'a>, Option<Block<'a>>),
    Goto(&'a str),
    Label(&'a str),
    FunDeclaration(&'a str, Vec<&'a str>, Block<'a>),
    
    //at the moment, the expr can only be a Expr::FunCall, otherwise
    //ambiguity arises (eg. 'end' getting parsed as Expr::Var("end") instead
    //of the end of a block)
    Expr(Expr<'a>)
}

#[derive(Debug)]
pub enum DataDef<'a> {
    Label(&'a str),
    Bytes(Vec<u8>)
}