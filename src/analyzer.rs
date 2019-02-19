use crate::stream::Stream;
use std::marker::PhantomData;

pub trait Analyzer {
    type Input;
    type Output;
    fn analyze(&self, stream: &mut Stream<Self::Input>) -> Option<Self::Output>;
    fn map<T, F: Fn(Self::Output) -> T>(self, f: F) -> Map<T, Self, F>
    where
        Self: Sized,
    {
        Map::new(f, self)
    }

    fn attempt(self) -> Try<Self>
    where
        Self: Sized,
    {
        Try::new(self)
    }
}

pub fn anyOne<T: Clone>() -> AnyOne<T> {
    AnyOne::new()
}

pub struct AnyOne<T: Clone>(PhantomData<T>);

impl<T: Clone> AnyOne<T> {
    pub fn new() -> Self {
        AnyOne(PhantomData)
    }
}

impl<T: Clone> Analyzer for AnyOne<T> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> Option<Self::Output> {
        let val = st.peak()?;
        st.add_pos(1);
        Some(val)
    }
}

pub struct Try<T: Analyzer>(T);

impl<T: Analyzer> Try<T> {
    pub fn new(x: T) -> Self {
        Try(x)
    }
}

impl<T: Analyzer> Analyzer for Try<T> {
    type Input = T::Input;
    type Output = T::Output;
    fn analyze(&self, st: &mut Stream<T::Input>) -> Option<T::Output> {
        let pos = st.pos();
        let res = self.0.analyze(st);
        if let None = res {
            st.set_pos(pos);
        }
        res
    }
}

pub struct Map<O, T: Analyzer, F: Fn(T::Output) -> O>(F, T, PhantomData<O>);

impl<O, T: Analyzer, F: Fn(T::Output) -> O> Map<O, T, F> {
    pub fn new(f: F, x: T) -> Self {
        Map(f, x, PhantomData)
    }
}

impl<O, T: Analyzer, F: Fn(T::Output) -> O> Analyzer for Map<O, T, F> {
    type Input = T::Input;
    type Output = O;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> Option<Self::Output> {
        Some(self.0(self.1.analyze(st)?))
    }
}

pub struct Or<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> Or<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Or(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> Analyzer for Or<A, B> {
    type Input = A::Input;
    type Output = B::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> Option<Self::Output> {
        match self.0.analyze(st) {
            None => self.1.analyze(st),
            x => x,
        }
    }
}
