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
pub struct AnalyzerError {
    pos: usize,
    unexpected: String,
    expecting: String,
}

impl AnalyzerError {
    pub fn new(pos: usize, unexpected: String, expecting: String) -> AnalyzerError {
        AnalyzerError {
            pos,
            unexpected,
            expecting,
        }
    }
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unexpected {} expecting {}",
            self.unexpected, self.expecting
        )
    }
}

impl error::Error for AnalyzerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub type AnalyzerResult<T> = Result<T, AnalyzerError>;

pub trait Analyzer {
    type Input;
    type Output;
    fn analyze(&self, stream: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output>;
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

    fn msg(self, msg: String) -> Msg<Self>
    where
        Self: Sized,
    {
        Msg::new(self, msg)
    }

    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not::new(self)
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
}

pub fn anyOne<T: Clone>() -> AnyOne<T> {
    AnyOne::new()
}

pub fn eof<T: Clone + Debug>() -> Eof<T> {
    Eof::new()
}

pub fn val<T: Clone, I>(x: T) -> Val<T, I> {
    Val::new(x)
}

pub fn token<T: Clone + Eq + Debug>(x: T) -> Token<T> {
    Token::new(x)
}

pub fn tokens<T: Clone + Eq + Debug>(x: Vec<T>) -> Tokens<T> {
    Tokens::new(x)
}

pub fn expect<T: Clone + Debug, F: Fn(&T) -> bool>(f: F) -> Expect<T, F> {
    Expect::new(f)
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let val = st.peak().ok_or(AnalyzerError::new(
            st.pos(),
            "eof".to_string(),
            "anyToken".to_string(),
        ))?;
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
    fn analyze(&self, st: &mut Stream<T::Input>) -> AnalyzerResult<T::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, _: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let mut res = Vec::new();
        for i in 0.. {
            if let Some(max) = self.2 {
                if i >= max {
                    break;
                }
            }

            match self.0.analyze(st) {
                Ok(x) => res.push(x),
                Err(e) => {
                    if let Some(min) = self.1 {
                        if res.len() < min {
                            return Err(e);
                        }
                    }
                    break;
                }
            }
        }

        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct Eof<T: Clone + Debug>(PhantomData<T>);

impl<T: Clone + Debug> Eof<T> {
    pub fn new() -> Self {
        Eof(PhantomData)
    }
}

impl<T: Clone + Debug> Analyzer for Eof<T> {
    type Input = T;
    type Output = ();
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        if let Some(x) = st.peak() {
            Err(AnalyzerError::new(
                st.pos(),
                format!("{:?}", x),
                "eof".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token<T: Clone + Eq + Debug>(T);

impl<T: Clone + Eq + Debug> Token<T> {
    pub fn new(x: T) -> Self {
        Token(x)
    }
}

impl<T: Clone + Eq + Debug> Analyzer for Token<T> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let res = st.peak().ok_or(AnalyzerError::new(
            st.pos(),
            "eof".to_string(),
            format!("{:?}", self.0),
        ))?;
        if res == self.0 {
            st.next();
            Ok(res)
        } else {
            Err(AnalyzerError::new(
                st.pos(),
                format!("{:?}", res),
                format!("{:?}", self.0),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens<T: Clone + Eq + Debug>(Vec<T>);

impl<T: Clone + Eq + Debug> Tokens<T> {
    pub fn new(x: Vec<T>) -> Self {
        Tokens(x)
    }
}

impl<T: Clone + Eq + Debug> Analyzer for Tokens<T> {
    type Input = T;
    type Output = Vec<T>;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let mut res = Vec::new();

        for x in self.0.iter() {
            let y = st.peak().ok_or(AnalyzerError::new(
                st.pos(),
                "eof".to_string(),
                format!("{:?}", x),
            ))?;
            if x.clone() == y {
                st.next();
                res.push(y);
            } else {
                return Err(AnalyzerError::new(
                    st.pos(),
                    format!("{:?}", y),
                    format!("{:?}", x),
                ));
            }
        }
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct Expect<T: Clone + Debug, F: Fn(&T) -> bool>(F, PhantomData<T>);

impl<T: Clone + Debug, F: Fn(&T) -> bool> Expect<T, F> {
    pub fn new(f: F) -> Self {
        Expect(f, PhantomData)
    }
}

impl<T: Clone + Debug, F: Fn(&T) -> bool> Analyzer for Expect<T, F> {
    type Input = T;
    type Output = T;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let x = st.peak().ok_or(AnalyzerError::new(
            st.pos(),
            "eof".to_string(),
            "???".to_string(),
        ))?;

        if self.0(&x) {
            st.next();
            Ok(x)
        } else {
            Err(AnalyzerError::new(
                st.pos(),
                format!("{:?}", x),
                "???".to_string(),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Msg<A: Analyzer>(A, String);

impl<A: Analyzer> Msg<A> {
    pub fn new(a: A, msg: String) -> Self {
        Msg(a, msg)
    }
}

impl<A: Analyzer> Analyzer for Msg<A> {
    type Input = A::Input;
    type Output = A::Output;
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        self.0.analyze(st).map_err(|mut e| {
            e.expecting = self.1.clone();
            e
        })
    }
}

#[derive(Clone, Debug)]
pub struct Not<A: Analyzer>(A);

impl<A: Analyzer> Not<A> {
    pub fn new(a: A) -> Self {
        Not(a)
    }
}

impl<A: Analyzer> Analyzer for Not<A> {
    type Input = A::Input;
    type Output = ();
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        let pos = st.pos();
        match self.0.analyze(st) {
            Ok(_) => Err(AnalyzerError::new(
                pos,
                "???".to_string(),
                "???".to_string(),
            )),
            Err(_) => Ok(()),
        }
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
    fn analyze(&self, st: &mut Stream<Self::Input>) -> AnalyzerResult<Self::Output> {
        match self.0.analyze(st) {
            Ok(x) => self.1(x).analyze(st),
            Err(e) => Err(e),
        }
    }
}
