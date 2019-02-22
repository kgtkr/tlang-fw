use crate::analyzer;
use crate::analyzer::{
    analyzer_func, any_one, eof, expect, fail, token, tokens, val, Analyzer, AnalyzerError,
    AnalyzerResult, Either,
};
use crate::stream::Stream;
use crate::token::{Keyword, Kind, Literal, NumLiteral, Symbol, Token};

pub fn string(s: &str) -> impl Analyzer<Input = char, Output = String> {
    tokens(s.chars().collect()).map(|x| x.into_iter().collect())
}

pub fn skip() -> impl Analyzer<Input = char, Output = ()> {
    let spaces = analyzer::or!(token(' '), token('\n'), token('\t'))
        .many()
        .with(val(()));
    let line_comment = string("//")
        .with(expect(|&x| x != '\n').many())
        .with(token('\n').optional())
        .with(val(()));
    fn block_comment_f(st: &mut Stream<char>) -> AnalyzerResult<()> {
        string("/*")
            .with(
                analyzer_func(|st| match (st.peak(), st.peak_index(1)) {
                    (Some('/'), Some('*')) => block_comment_f(st),
                    (Some('*'), Some('/')) => fail().analyze(st),
                    _ => any_one().with(val(())).analyze(st),
                })
                .many(),
            )
            .with(string("*/"))
            .with(val(()))
            .analyze(st)
    }
    let block_comment = analyzer_func(block_comment_f);

    let comment = line_comment.attempt().or(block_comment);

    spaces.or(comment).many().with(val(()))
}

pub fn ident_str() -> impl Analyzer<Input = char, Output = String> {
    expect::<char, _>(|&c| c.is_ascii_alphabetic())
        .and(expect::<char, _>(|&c| c.is_ascii_alphanumeric() || c == '_').many())
        .map(|(x, mut xs)| {
            xs.insert(0, x);
            xs.into_iter().collect::<String>()
        })
}

pub fn num_literal() -> impl Analyzer<Input = char, Output = NumLiteral> {
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
                    None | Some("f64") => {
                        if let Ok(x) = s.parse::<f64>() {
                            Either::Right(val(NumLiteral::F64(x)))
                        } else {
                            Either::Left(fail())
                        }
                    }
                    Some("f32") => {
                        if let Ok(x) = s.parse::<f32>() {
                            Either::Right(val(NumLiteral::F32(x)))
                        } else {
                            Either::Left(fail())
                        }
                    }
                    _ => Either::Left(fail()),
                }
            } else {
                match suffix {
                    None | Some("i32") => {
                        if let Ok(x) = s1.parse::<i32>() {
                            Either::Right(val(NumLiteral::I32(x)))
                        } else {
                            Either::Left(fail())
                        }
                    }
                    Some("i64") => {
                        if let Ok(x) = s1.parse::<i64>() {
                            Either::Right(val(NumLiteral::I64(x)))
                        } else {
                            Either::Left(fail())
                        }
                    }
                    _ => Either::Left(fail()),
                }
            }
        })
}

pub fn hex_char(len: usize) -> impl Analyzer<Input = char, Output = char> {
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

pub fn lexer() -> impl Analyzer<Input = char, Output = Vec<Token>> {
    skip().with(one_token()).many1().skip(skip()).skip(eof())
}

pub fn one_token() -> impl Analyzer<Input = char, Output = Token> {
    analyzer_func(|st| {
        let pos = st.pos();
        let kind = kind().analyze(st)?;
        let len = st.pos() - pos;
        Ok(Token { pos, kind, len })
    })
}

pub fn kind() -> impl Analyzer<Input = char, Output = Kind> {
    analyzer::or!(
        ident_or_keyword(),
        symbol().map(Kind::Symbol),
        literal().map(Kind::Literal)
    )
}

pub fn literal() -> impl Analyzer<Input = char, Output = Literal> {
    analyzer::or!(
        char_literal().map(Literal::Char),
        string_literal().map(Literal::String),
        num_literal().map(Literal::Num)
    )
}

pub fn literal_char(lit: char) -> impl Analyzer<Input = char, Output = char> {
    analyzer_func(move |st| {
        let c = any_one().analyze(st)?;
        if c == '\\' {
            let pos = st.pos();
            let c = any_one().analyze(st)?;
            match c {
                't' => Ok('\t'),
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                '\\' => Ok('\\'),
                c if c == lit => Ok(c),
                'x' => hex_char(2).analyze(st),
                'u' => hex_char(4).analyze(st),
                'U' => hex_char(8).analyze(st),
                c => Err(AnalyzerError::new(
                    pos,
                    "t or n or r or \\ or x or u or U".to_string(),
                    c.to_string(),
                )),
            }
        } else {
            Ok(c)
        }
    })
}

pub fn char_literal() -> impl Analyzer<Input = char, Output = char> {
    token('\'').with(literal_char('\'')).skip(token('\''))
}

pub fn string_literal() -> impl Analyzer<Input = char, Output = String> {
    token('\"')
        .with(
            literal_char('\"')
                .many()
                .map(|x| x.into_iter().collect::<String>()),
        )
        .skip(token('\"'))
}

pub fn ident_or_keyword() -> impl Analyzer<Input = char, Output = Kind> {
    analyzer_func(|st| {
        let s = ident_str().analyze(st)?;
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

pub fn symbol() -> impl Analyzer<Input = char, Output = Symbol> {
    analyzer::or!(
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
