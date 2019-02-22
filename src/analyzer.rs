use crate::stream::Stream;
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

pub macro or {
  ($x:expr) => {
    $x
  },
  ($x:expr, $($xs:tt)+) => {
    $x.or(or!($($xs)+))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorExpect<T> {
    Any,
    Eof,
    Token(T),
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalyzerError<T> {
    pos: usize,
    unexpected: Option<T>,
    expecting: ErrorExpect<T>,
}

impl<T> AnalyzerError<T> {
    pub fn new(pos: usize, unexpected: Option<T>, expecting: ErrorExpect<T>) -> AnalyzerError<T> {
        AnalyzerError {
            pos,
            unexpected,
            expecting,
        }
    }
}

impl<T: Debug> fmt::Display for AnalyzerError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unexpected {:?} expecting {:?}",
            self.unexpected, self.expecting
        )
    }
}

impl<T: Debug> error::Error for AnalyzerError<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub type AnalyzerResult<O, I> = Result<O, AnalyzerError<I>>;

pub trait Analyzer {
    type Input;
    type Output;
    fn analyze(
        &self,
        stream: &mut Stream<Self::Input>,
    ) -> AnalyzerResult<Self::Output, Self::Input>;
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

    fn val<T: Clone>(self, x: T) -> With<Self, Val<T, Self::Input>>
    where
        Self: Sized,
    {
        self.with(val(x))
    }

    fn with<T: Analyzer<Input = Self::Input>>(self, x: T) -> With<Self, T>
    where
        Self: Sized,
    {
        With::new(self, x)
    }

    fn skip<T: Analyzer<Input = Self::Input>>(self, x: T) -> Skip<Self, T>
    where
        Self: Sized,
    {
        Skip::new(self, x)
    }

    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        Optional::new(self)
    }

    fn many(self) -> Loop<Self>
    where
        Self: Sized,
    {
        Loop::new(self, None, None)
    }

    fn many1(self) -> Loop<Self>
    where
        Self: Sized,
    {
        Loop::new(self, Some(1), None)
    }

    fn many_n(self, n: usize) -> Loop<Self>
    where
        Self: Sized,
    {
        Loop::new(self, Some(n), Some(n))
    }

    fn msg(self, msg: ErrorExpect<Self::Input>) -> Msg<Self>
    where
        Self: Sized,
        Self::Input: Clone,
    {
        Msg::new(self, msg)
    }

    fn then<F: Fn(Self::Output) -> B, B: Analyzer<Input = Self::Input>>(
        self,
        f: F,
    ) -> Then<Self, F, B>
    where
        Self: Sized,
    {
        Then::new(self, f)
    }

    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<A: Analyzer> Analyzer for Box<A> {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        (**self).analyze(st)
    }
}

impl<A: Analyzer> Analyzer for &A {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        (**self).analyze(st)
    }
}

impl<A: Analyzer> Analyzer for &mut A {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        (**self).analyze(st)
    }
}

pub fn any_one<T: Clone>() -> AnyOne<T> {
    AnyOne::new()
}

pub fn eof<T: Clone>() -> Eof<T> {
    Eof::new()
}

pub fn val<T: Clone, I>(x: T) -> Val<T, I> {
    Val::new(x)
}

pub fn token<T: Clone + Eq>(x: T) -> Token<T> {
    Token::new(x)
}

pub fn tokens<T: Clone + Eq>(x: Vec<T>) -> Tokens<T> {
    Tokens::new(x)
}

pub fn expect<T: Clone, F: Fn(&T) -> bool>(f: F) -> Expect<T, F> {
    Expect::new(f)
}

pub fn analyzer_func<F: Fn(&mut Stream<A>) -> AnalyzerResult<B, A>, A, B>(
    f: F,
) -> AnalyzerFunc<F, A, B> {
    AnalyzerFunc::new(f)
}

pub fn fail<A: Clone, B>() -> Fail<A, B> {
    Fail::new()
}

#[derive(Clone, Debug)]
pub struct AnyOne<T: Clone>(PhantomData<T>);

impl<T: Clone> AnyOne<T> {
    pub fn new() -> Self {
        AnyOne(PhantomData)
    }
}

impl<T: Clone> Analyzer for AnyOne<T> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let val = st
            .peak()
            .ok_or(AnalyzerError::new(st.pos(), None, ErrorExpect::Any))?;
        st.next();
        Ok(val)
    }
}

#[derive(Clone, Debug)]
pub struct Attempt<T: Analyzer>(T);

impl<T: Analyzer> Attempt<T> {
    pub fn new(x: T) -> Self {
        Attempt(x)
    }
}

impl<T: Analyzer> Analyzer for Attempt<T> {
    type Input = T::Input;
    type Output = T::Output;
    fn analyze(&self, st: &mut Stream<T::Input>) -> AnalyzerResult<T::Output, T::Input> {
        let pos = st.pos();
        let res = self.0.analyze(st);
        if let Err(_) = res {
            st.set_pos(pos);
        }
        res
    }
}

#[derive(Clone, Debug)]
pub struct Map<O, T: Analyzer, F: Fn(T::Output) -> O>(F, T, PhantomData<O>);

impl<O, T: Analyzer, F: Fn(T::Output) -> O> Map<O, T, F> {
    pub fn new(f: F, x: T) -> Self {
        Map(f, x, PhantomData)
    }
}

impl<O, T: Analyzer, F: Fn(T::Output) -> O> Analyzer for Map<O, T, F> {
    type Input = T::Input;
    type Output = O;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        Ok(self.0(self.1.analyze(st)?))
    }
}

#[derive(Clone, Debug)]
pub struct Val<T: Clone, I>(T, PhantomData<I>);

impl<T: Clone, I> Val<T, I> {
    pub fn new(x: T) -> Self {
        Val(x, PhantomData)
    }
}

impl<T: Clone, I> Analyzer for Val<T, I> {
    type Input = I;
    type Output = T;
    fn analyze(&self, _: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        Ok(self.0.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Or<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> Or<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Or(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> Analyzer for Or<A, B> {
    type Input = A::Input;
    type Output = B::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        match self.0.analyze(st) {
            Err(_) => self.1.analyze(st),
            x => x,
        }
    }
}

#[derive(Clone, Debug)]
pub struct And<A: Analyzer, B: Analyzer<Input = A::Input>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input>> And<A, B> {
    pub fn new(a: A, b: B) -> Self {
        And(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input>> Analyzer for And<A, B> {
    type Input = A::Input;
    type Output = (A::Output, B::Output);
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        Ok((self.0.analyze(st)?, self.1.analyze(st)?))
    }
}

#[derive(Clone, Debug)]
pub struct With<A: Analyzer, B: Analyzer<Input = A::Input>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input>> With<A, B> {
    pub fn new(a: A, b: B) -> Self {
        With(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input>> Analyzer for With<A, B> {
    type Input = A::Input;
    type Output = B::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        self.0.analyze(st)?;
        self.1.analyze(st)
    }
}

#[derive(Clone, Debug)]
pub struct Skip<A: Analyzer, B: Analyzer<Input = A::Input>>(A, B);

impl<A: Analyzer, B: Analyzer<Input = A::Input>> Skip<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Skip(a, b)
    }
}

impl<A: Analyzer, B: Analyzer<Input = A::Input>> Analyzer for Skip<A, B> {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let res = self.0.analyze(st)?;
        self.1.analyze(st)?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct Optional<A: Analyzer>(A);

impl<A: Analyzer> Optional<A> {
    pub fn new(a: A) -> Self {
        Optional(a)
    }
}

impl<A: Analyzer> Analyzer for Optional<A> {
    type Input = A::Input;
    type Output = Option<A::Output>;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        Ok(self.0.analyze(st).ok())
    }
}

#[derive(Clone, Debug)]
pub struct Loop<A: Analyzer>(A, Option<usize>, Option<usize>);

impl<A: Analyzer> Loop<A> {
    pub fn new(a: A, x: Option<usize>, y: Option<usize>) -> Self {
        Loop(a, x, y)
    }
}

impl<A: Analyzer> Analyzer for Loop<A> {
    type Input = A::Input;
    type Output = Vec<A::Output>;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let mut res = Vec::new();
        for i in 0.. {
            if let Some(max) = self.2 {
                if i >= max {
                    break;
                }
            }

            let pos = st.pos();
            match self.0.analyze(st) {
                Ok(x) => res.push(x),
                Err(e) => {
                    if let Some(min) = self.1 {
                        if res.len() < min {
                            return Err(e);
                        }
                    }
                    if st.pos() != pos {
                        return Err(e);
                    }
                    break;
                }
            }
        }

        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct Eof<T: Clone>(PhantomData<T>);

impl<T: Clone> Eof<T> {
    pub fn new() -> Self {
        Eof(PhantomData)
    }
}

impl<T: Clone> Analyzer for Eof<T> {
    type Input = T;
    type Output = ();
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        if let Some(x) = st.peak() {
            Err(AnalyzerError::new(st.pos(), Some(x), ErrorExpect::Eof))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token<T: Clone + Eq>(T);

impl<T: Clone + Eq> Token<T> {
    pub fn new(x: T) -> Self {
        Token(x)
    }
}

impl<T: Clone + Eq> Analyzer for Token<T> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let res = st.peak().ok_or(AnalyzerError::new(
            st.pos(),
            None,
            ErrorExpect::Token(self.0.clone()),
        ))?;
        if res == self.0 {
            st.next();
            Ok(res)
        } else {
            Err(AnalyzerError::new(
                st.pos(),
                Some(res),
                ErrorExpect::Token(self.0.clone()),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens<T: Clone + Eq>(Vec<T>);

impl<T: Clone + Eq> Tokens<T> {
    pub fn new(x: Vec<T>) -> Self {
        Tokens(x)
    }
}

impl<T: Clone + Eq> Analyzer for Tokens<T> {
    type Input = T;
    type Output = Vec<T>;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let mut res = Vec::new();

        for x in self.0.iter() {
            let y = st.peak().ok_or(AnalyzerError::new(
                st.pos(),
                None,
                ErrorExpect::Token(x.clone()),
            ))?;
            if x.clone() == y {
                st.next();
                res.push(y);
            } else {
                return Err(AnalyzerError::new(
                    st.pos(),
                    Some(y),
                    ErrorExpect::Token(x.clone()),
                ));
            }
        }
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct Expect<T: Clone, F: Fn(&T) -> bool>(F, PhantomData<T>);

impl<T: Clone, F: Fn(&T) -> bool> Expect<T, F> {
    pub fn new(f: F) -> Self {
        Expect(f, PhantomData)
    }
}

impl<T: Clone, F: Fn(&T) -> bool> Analyzer for Expect<T, F> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        let x = st
            .peak()
            .ok_or(AnalyzerError::new(st.pos(), None, ErrorExpect::Unknown))?;

        if self.0(&x) {
            st.next();
            Ok(x)
        } else {
            Err(AnalyzerError::new(st.pos(), Some(x), ErrorExpect::Unknown))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Msg<A: Analyzer>(A, ErrorExpect<A::Input>);

impl<A: Analyzer> Msg<A>
where
    A::Input: Clone,
{
    pub fn new(a: A, msg: ErrorExpect<A::Input>) -> Self {
        Msg(a, msg)
    }
}

impl<A: Analyzer> Analyzer for Msg<A>
where
    A::Input: Clone,
{
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        self.0.analyze(st).map_err(|mut e| {
            e.expecting = self.1.clone();
            e
        })
    }
}

#[derive(Clone, Debug)]
pub struct Then<A: Analyzer, F: Fn(A::Output) -> B, B: Analyzer<Input = A::Input>>(
    A,
    F,
    PhantomData<B>,
);

impl<A: Analyzer, F: Fn(A::Output) -> B, B: Analyzer<Input = A::Input>> Then<A, F, B> {
    pub fn new(a: A, f: F) -> Self {
        Then(a, f, PhantomData)
    }
}

impl<A: Analyzer, F: Fn(A::Output) -> B, B: Analyzer<Input = A::Input>> Analyzer for Then<A, F, B> {
    type Input = A::Input;
    type Output = B::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        match self.0.analyze(st) {
            Ok(x) => self.1(x).analyze(st),
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnalyzerFunc<F: Fn(&mut Stream<A>) -> AnalyzerResult<B, A>, A, B>(
    F,
    PhantomData<(A, B)>,
);

impl<F: Fn(&mut Stream<A>) -> AnalyzerResult<B, A>, A, B> AnalyzerFunc<F, A, B> {
    pub fn new(f: F) -> Self {
        AnalyzerFunc(f, PhantomData)
    }
}

impl<F: Fn(&mut Stream<A>) -> AnalyzerResult<B, A>, A, B> Analyzer for AnalyzerFunc<F, A, B> {
    type Input = A;
    type Output = B;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        self.0(st)
    }
}

#[derive(Clone, Debug)]
pub struct Fail<A: Clone, B>(PhantomData<(A, B)>);

impl<A: Clone, B> Fail<A, B> {
    pub fn new() -> Self {
        Fail(PhantomData)
    }
}

impl<A: Clone, B> Analyzer for Fail<A, B> {
    type Input = A;
    type Output = B;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        Err(AnalyzerError::new(
            st.pos(),
            st.peak().map(Some).unwrap_or(None),
            ErrorExpect::Unknown,
        ))
    }
}

#[derive(Clone, Debug)]
pub enum Either<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> {
    Right(A),
    Left(B),
}

impl<A: Analyzer, B: Analyzer<Input = A::Input, Output = A::Output>> Analyzer for Either<A, B> {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output, Self::Input> {
        match self {
            Either::Right(x) => x.analyze(st),
            Either::Left(x) => x.analyze(st),
        }
    }
}
