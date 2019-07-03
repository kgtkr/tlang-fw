use crate::analyzer::{token, val, Analyzer};
use crate::ast::Expr;
use crate::token::{Kind, Symbol, Token};

pub fn expr() -> impl Analyzer<Input = Kind, Output = Expr> {
    unimplemented!();
    val(Expr::I32Literal(1))
}

pub fn block() -> impl Analyzer<Input = Kind, Output = Expr> {
    token(Kind::Symbol(Symbol::OpenBrace))
        .with(
            expr()
                .skip(token(Kind::Symbol(Symbol::Semicolon)))
                .attempt()
                .many(),
        )
        .and(expr().optional())
        .map(|(a, b)| Expr::Block(a, Box::new(b)))
}
