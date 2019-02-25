pub type Ident = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    StructLiteral(Ident, Vec<(Ident, Expr)>),
    I32Literal(i32),
    I64Literal(i64),
    F32Literal(f32),
    F64Literal(f64),
    StringLiteral(String),
    ArrayLiteral(Type, Box<Expr>),
    BoolLiteral(bool),
    CharLiteral(char),
    Var(Ident),
    Not(Box<Expr>),
    Plus(Box<Expr>),
    Minus(Box<Expr>),
    Member(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>, Box<Option<Expr>>),
    Let(Ident, Box<Expr>),
    If(Box<(Expr, Expr)>, Vec<(Expr, Expr)>, Box<Option<Expr>>),
    While(Box<Expr>, Box<Expr>),
    Return(Box<Option<Expr>>),
    Set(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    Lambda(Vec<Ident>, Vec<(Ident, Type)>, Type, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    RefType(RefType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RefType {
    String,
    Array(Box<Type>),
    Struct(Ident),
    Func(Vec<Type>, Box<Option<Type>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncDef(Ident, Vec<(Ident, Type)>, Option<Type>);

#[derive(Clone, Debug, PartialEq)]
pub enum Member {
    Struct(Ident, Vec<(Ident, Type)>),
    Func(FuncDef, Expr),
    ExternFun(FuncDef, String, String),
}

pub type Module = Vec<Member>;
