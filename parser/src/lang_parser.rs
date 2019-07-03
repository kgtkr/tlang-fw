
use ast::ast::Expr;
use crate::parser::{token, val, Parser};
use crate::token::{Kind, Symbol};

pub fn expr() -> impl Parser<Input = Kind, Output = Expr> {
        unimplemented!();
        val(Expr::I32Literal(1))
}

pub fn block() -> impl Parser<Input = Kind, Output = Expr> {
        token(Kind::Symbol(Symbol::OpenBrace))
                .with(expr()
                        .skip(token(Kind::Symbol(Symbol::Semicolon)))
                        .attempt()
                        .many())
                .and(expr().optional())
                .map(|(a, b)| Expr::Block(a, Box::new(b)))
}
