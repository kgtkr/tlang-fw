use crate::parser;
use crate::parser::{
    analyzer_func, any_one, eof, expect, fail, token, tokens, val, Either, Fail, Parser,
    ParserError, ParserResult, Val,
};
use crate::stream::Stream;
use token::token::{Keyword, Kind, Literal, NumLiteral, Symbol, Token};

pub fn string(s: &str) -> impl Parser<Input = char, Output = String> {
    tokens(s.chars().collect()).map(|x| x.into_iter().collect())
}

pub fn space() -> impl Parser<Input = char, Output = ()> {
    or!(token(' '), token('\n'), token('\t')).with(val(()))
}

pub fn line_comment() -> impl Parser<Input = char, Output = ()> {
    string("//")
        .with(expect(|&x| x != '\n').many())
        .with(token('\n').optional())
        .with(val(()))
}

pub fn block_comment() -> impl Parser<Input = char, Output = ()> {
    analyzer_func(|st| {
        string("/*")
            .with(
                analyzer_func(|st| match (st.peak(), st.peak_index(1)) {
                    (Some('/'), Some('*')) => block_comment().parse(st),
                    (Some('*'), Some('/')) => fail().parse(st),
                    _ => any_one().with(val(())).parse(st),
                })
                .many(),
            )
            .with(string("*/"))
            .with(val(()))
            .parse(st)
    })
}

pub fn comment() -> impl Parser<Input = char, Output = ()> {
    line_comment().attempt().or(block_comment())
}

pub fn skip() -> impl Parser<Input = char, Output = ()> {
    space().or(comment())
}

pub fn ident_str() -> impl Parser<Input = char, Output = String> {
    expect::<char, _>(|&c| c.is_ascii_alphabetic())
        .and(expect::<char, _>(|&c| c.is_ascii_alphanumeric() || c == '_').many())
        .map(|(x, mut xs)| {
            xs.insert(0, x);
            xs.into_iter().collect::<String>()
        })
}

pub fn num_literal() -> impl Parser<Input = char, Output = NumLiteral> {
    fn parse<T: std::str::FromStr, F: Fn(T) -> NumLiteral>(
        s: String,
        f: F,
    ) -> Either<Val<NumLiteral, char>, Fail<char, NumLiteral>> {
        s.parse::<T>()
            .map(|x| Either::Right(val(f(x))))
            .unwrap_or(Either::Left(fail()))
    }

    let num = expect::<char, _>(|&c| c.is_ascii_digit())
        .many1()
        .map(|x| x.into_iter().collect::<String>());
    num.clone()
        .and(token('.').and(num).optional())
        .and(ident_str().optional())
        .then(|((s1, dot_num), suffix)| {
            let suffix = suffix.as_ref().map(|x| x.as_str());
            if let Some((_, s2)) = dot_num {
                let s = format!("{}.{}", s1, s2);
                match suffix {
                    None | Some("f64") => parse::<_, _>(s, NumLiteral::F64),
                    Some("f32") => parse::<_, _>(s, NumLiteral::F32),
                    _ => Either::Left(fail()),
                }
            } else {
                match suffix {
                    None | Some("i32") => parse::<_, _>(s1, NumLiteral::I32),
                    Some("i64") => parse::<_, _>(s1, NumLiteral::I64),
                    Some("f32") => parse::<_, _>(s1, NumLiteral::F32),
                    Some("f64") => parse::<_, _>(s1, NumLiteral::F64),
                    _ => Either::Left(fail()),
                }
            }
        })
}

pub fn hex_char(len: usize) -> impl Parser<Input = char, Output = char> {
    expect::<char, _>(|&x| x.is_ascii_digit() || ('a' <= x && x <= 'f') || ('A' <= x && x <= 'F'))
        .map(|x| x.to_ascii_lowercase())
        .many_n(len)
        .map(|x| {
            u32::from_str_radix(&x.into_iter().collect::<String>(), 16)
                .map(|x| std::char::from_u32(x))
                .unwrap()
        })
        .then(|x| match x {
            Some(x) => Either::Right(val(x)),
            None => Either::Left(fail()),
        })
}

pub fn lexer() -> impl Parser<Input = char, Output = Vec<Token>> {
    skip()
        .map(|_| None)
        .or(one_token().map(Some))
        .many()
        .map(|x| x.into_iter().filter_map(|x| x).collect::<Vec<_>>())
        .skip(eof())
}

pub fn one_token() -> impl Parser<Input = char, Output = Token> {
    analyzer_func(|st| {
        let pos = st.pos();
        let kind = kind().parse(st)?;
        let len = st.pos() - pos;
        Ok(Token { pos, kind, len })
    })
}

pub fn kind() -> impl Parser<Input = char, Output = Kind> {
    or!(
        ident_or_keyword(),
        symbol().map(Kind::Symbol),
        literal().map(Kind::Literal)
    )
}

pub fn literal() -> impl Parser<Input = char, Output = Literal> {
    or!(
        char_literal().map(Literal::Char),
        string_literal().map(Literal::String),
        num_literal().map(Literal::Num)
    )
}

pub fn literal_char(lit: char) -> impl Parser<Input = char, Output = char> {
    or!(
        token('\\').with(or!(
            token('t').val('\t'),
            token('n').val('\n'),
            token('r').val('\r'),
            token('\\').val('\\'),
            token(lit).val(lit),
            token('x').with(hex_char(2)),
            token('u').with(hex_char(4)),
            token('U').with(hex_char(8))
        )),
        expect(move |&x| x != lit)
    )
}

pub fn char_literal() -> impl Parser<Input = char, Output = char> {
    token('\'').with(literal_char('\'')).skip(token('\''))
}

pub fn string_literal() -> impl Parser<Input = char, Output = String> {
    token('\"')
        .with(
            literal_char('\"')
                .many()
                .map(|x| x.into_iter().collect::<String>()),
        )
        .skip(token('\"'))
}

pub fn ident_or_keyword() -> impl Parser<Input = char, Output = Kind> {
    analyzer_func(|st| {
        let s = ident_str().parse(st)?;
        Ok(match s.as_ref() {
            "i32" => Kind::Keyword(Keyword::I32),
            "i64" => Kind::Keyword(Keyword::I64),
            "F32" => Kind::Keyword(Keyword::F32),
            "F64" => Kind::Keyword(Keyword::F64),
            "string" => Kind::Keyword(Keyword::String),
            "bool" => Kind::Keyword(Keyword::Bool),
            "char" => Kind::Keyword(Keyword::Char),
            "true" => Kind::Keyword(Keyword::True),
            "false" => Kind::Keyword(Keyword::False),
            "let" => Kind::Keyword(Keyword::Let),
            "if" => Kind::Keyword(Keyword::If),
            "while" => Kind::Keyword(Keyword::While),
            "return" => Kind::Keyword(Keyword::Return),
            "struct" => Kind::Keyword(Keyword::Struct),
            "fun" => Kind::Keyword(Keyword::Fun),
            "extern" => Kind::Keyword(Keyword::Extern),
            "for" => Kind::Keyword(Keyword::For),
            s => Kind::Ident(s.to_string()),
        })
    })
}

pub fn symbol() -> impl Parser<Input = char, Output = Symbol> {
    or!(
        token('.').with(val(Symbol::Dot)),
        token(',').with(val(Symbol::Comma)),
        token(':').with(val(Symbol::Colon)),
        token(';').with(val(Symbol::Semicolon)),
        token('(').with(val(Symbol::OpenParent)),
        token(')').with(val(Symbol::CloseParent)),
        token('[').with(val(Symbol::OpenBracket)),
        token(']').with(val(Symbol::CloseBracket)),
        token('{').with(val(Symbol::OpenBrace)),
        token('}').with(val(Symbol::CloseBrace)),
        string("!=").with(val(Symbol::Ne)).attempt(),
        token('!').with(val(Symbol::Not)),
        token('+').with(val(Symbol::Add)),
        token('-').with(val(Symbol::Sub)),
        string("**").with(val(Symbol::Pow)).attempt(),
        token('*').with(val(Symbol::Mul)),
        token('/').with(val(Symbol::Div)),
        token('%').with(val(Symbol::Mod)),
        string("&&").with(val(Symbol::And)).attempt(),
        token('&').with(val(Symbol::BitAnd)),
        string("||").with(val(Symbol::Or)).attempt(),
        token('|').with(val(Symbol::BitOr)),
        token('^').with(val(Symbol::BitXor)),
        string("<=").with(val(Symbol::Lte)).attempt(),
        token('<').with(val(Symbol::Lt)),
        string(">=").with(val(Symbol::Gte)).attempt(),
        token('>').with(val(Symbol::Gt)),
        string("==").with(val(Symbol::Eq)).attempt(),
        token('=').with(val(Symbol::Assign))
    )
}
