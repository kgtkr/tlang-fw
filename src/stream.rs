struct Stream<T>(Vec<T>, usize);

trait Analyzer {
    type Input;
    type Output;
    fn analyze(&self, stream: &mut Stream<Self::Input>) -> Option<Self::Output>;
}

struct AnyOne<T: Clone>(std::marker::PhantomData<T>);

impl<T: Clone> AnyOne<T> {
    fn new() -> Self {
        AnyOne(std::marker::PhantomData)
    }
}

impl<T: Clone> Analyzer for AnyOne<T> {
    type Input = T;
    type Output = T;
    fn analyze(&self, Stream(data, pos): &mut Stream<T>) -> Option<T> {
        let val = data.get(*pos).cloned()?;
        *pos += 1;
        Some(val)
    }
}

struct Try<T: Analyzer>(T);

impl<T: Analyzer> Try<T> {
    fn new(x: T) -> Try<T> {
        Try(x)
    }
}

impl<T: Analyzer> Analyzer for Try<T> {
    type Input = T::Input;
    type Output = T::Output;
    fn analyze(&self, st: &mut Stream<T::Input>) -> Option<T::Output> {
        let pos = st.1;
        let res = self.0.analyze(st);
        if let None = res {
            st.1 = pos;
        }
        res
    }
}
