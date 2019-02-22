use std::fs;
use tkr_lang::{analyzer::Analyzer, lexer::lexer, stream::Stream};

fn main() {
    let s = fs::read_to_string("test").unwrap();
    let mut st = Stream::new(s.chars().collect());
    println!("{:?}", lexer().analyze(&mut st));
}
