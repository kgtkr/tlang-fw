use crate::analyzer;
use crate::analyzer::{
    analyzer_func, anyOne, eof, expect, fail, token, tokens, val, Analyzer, AnalyzerResult,
};
use crate::stream::Stream;
use crate::token::Token;

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

pub fn ident() -> impl Analyzer<Input = char, Output = String> {
    expect::<char, _>(|&c| c.is_ascii_alphabetic())
        .and(expect::<char, _>(|&c| c.is_ascii_alphanumeric() || c == '_').many())
        .map(|(x, mut xs)| {
            xs.insert(0, x);
            xs.into_iter().collect::<String>()
        })
}
