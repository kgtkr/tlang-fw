#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    kind: Kind,
    pos: usize,
    len: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Keyword(Keyword),
    Ident(String),
    Literal(Literal),
    Symbol(Symbol),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Char(char),
    String(String),
    NumLiteral(NumLiteral),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumLiteral {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    I32,
    I64,
    F32,
    F64,
    String,
    Bool,
    Char,
    True,
    False,
    Let,
    If,
    While,
    Return,
    Struct,
    Fun,
    Extern,
    For,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    Dot,
    Comma,
    Colon,
    Semicolon,
    OpenParent,
    CloseParent,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Pow,
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Assign,
}
