pub enum AST {
}

pub type Ident<'a> = &'a str;

type VarTy<'a> = (Ident<'a>, Typ);

pub enum Typ {
    String
}

pub enum Descriptor<'a> {
    Output,
    Input,
    File(Ident<'a>)
}

pub enum Statement<'a> {
    WriteLn(Descriptor<'a>, Vec<Expr<'a>>),
    ReadLn(Descriptor<'a>, Vec<Expr<'a>>)
}

pub enum Expr<'a> {
    ConstStr(&'a str)
}

pub struct Program<'a> {
    name: &'a str,
    variables: Vec<VarTy<'a>>,
    body: Vec<Statement<'a>>
}
