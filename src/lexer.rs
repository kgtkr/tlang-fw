use crate::analyzer;
use crate::analyzer::{
    analyzer_func, anyOne, eof, expect, fail, token, tokens, val, Analyzer, AnalyzerResult,
};
use crate::stream::Stream;
use crate::token::{Keyword, Kind, Symbol, Token};

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
                    _ => anyOne().with(val(())).analyze(st),
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
