struct Stream<I> {
    data: Vec<I>,
    pos: usize,
}

trait Analyzer<I, O> {
    fn analyze(stream: &mut Stream<I>) -> O;
}
