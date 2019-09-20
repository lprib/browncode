pub type Block<'a> = Vec<Line<'a>>;

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
    //todo mod
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
    FunDeclaration(&'a str, Vec<&'a str>, Block<'a>)
}