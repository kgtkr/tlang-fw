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

    fn attempt(self) -> Attempt<Self>
    where
        Self: Sized,
    {
        Attempt::new(self)
    }

    fn or<T: Analyzer<Input = Self::Input, Output = Self::Output>>(self, x: T) -> Or<Self, T>
    where
        Self: Sized,
    {
        Or::new(self, x)
    }

    fn and<T: Analyzer<Input = Self::Input>>(self, x: T) -> And<Self, T>
    where
        Self: Sized,
    {
        And::new(self, x)
    }

    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        Optional::new(self)
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

pub struct Attempt<T: Analyzer>(T);

impl<T: Analyzer> Attempt<T> {
    pub fn new(x: T) -> Self {
        Attempt(x)
    }
}

impl<T: Analyzer> Analyzer for Attempt<T> {
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

pub struct And<A: Analyzer, B: Analyzer<Input = A::Input>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input>> And<A, B> {
    pub fn new(a: A, b: B) -> Self {
        And(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input>> Analyzer for And<A, B> {
    type Input = A::Input;
    type Output = B::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> Option<Self::Output> {
        self.0.analyze(st)?;
        self.1.analyze(st)
    }
}

pub struct Optional<A: Analyzer>(A);

impl<A: Analyzer> Optional<A> {
    pub fn new(a: A) -> Self {
        Optional(a)
    }
}

impl<A: Analyzer> Analyzer for Optional<A> {
    type Input = A::Input;
    type Output = Option<A::Output>;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> Option<Self::Output> {
        Some(self.0.analyze(st))
    }
}
