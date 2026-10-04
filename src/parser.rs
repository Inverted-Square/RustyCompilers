#![allow(unused)]

use crate::monadic_do;
use crate::monadic_type_classes::{Alternative, Monad};
use family_member_derive::FamilyMember;
use std::ops::BitOr;

type Matcher<T> = dyn FnOnce(&str) -> Option<(T, &str)>;

#[derive(FamilyMember)]
pub struct Parser<T>(Box<Matcher<T>>);

impl<T: 'static> Parser<T> {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce(&str) -> Option<(T, &str)> + 'static,
    {
        Self(Box::new(f))
    }

    pub fn parse(self, src: &str) -> Option<(T, &str)> {
        let Self(p) = self;
        p(src)
    }

    pub fn satisfy<F>(self, cond: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        monadic_do! {
            t <- self;
            if cond(&t) { Parser::ret(t) } else { parsers::fail() }
        }
    }
}

mod monadic_parser {
    use crate::monadic_type_classes::{Alternative, Applicative, Functor, Monad};
    use crate::parser::Parser;

    impl<A: 'static> Functor<A> for Parser<A> {
        fn fmap<B: 'static, F>(self, f: F) -> Parser<B>
        where
            F: FnOnce(A) -> B + 'static,
        {
            Parser::new(|src| self.parse(src).map(|(a, new_src)| (f(a), new_src)))
        }
    }

    impl<A: 'static> Applicative<A> for Parser<A> {
        fn pure(a: A) -> Self {
            Parser::new(|src| Some((a, src)))
        }

        fn apply<B: 'static, F>(self, pf: Parser<F>) -> Parser<B>
        where
            F: FnOnce(A) -> B + 'static,
        {
            Parser::new(|src| {
                pf.parse(src)
                    .and_then(|(f, new_src)| self.fmap(f).parse(new_src))
            })
        }
    }

    impl<A: 'static> Monad<A> for Parser<A> {
        fn bind<B: 'static, F>(self, f: F) -> Parser<B>
        where
            F: FnOnce(A) -> Parser<B> + 'static,
        {
            Parser::new(|src| self.parse(src).and_then(|(a, new_src)| f(a).parse(new_src)))
        }
    }

    impl<A: 'static> Alternative<A> for Parser<A> {
        fn empty() -> Self {
            Parser::new(|_| None)
        }

        fn alternative(self, pa: Self) -> Self {
            Parser::new(|src| self.parse(src).or_else(|| pa.parse(src)))
        }
    }
}

impl<T: 'static> BitOr for Parser<T> {
    type Output = Parser<T>;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.alternative(rhs)
    }
}

pub mod parsers {
    use crate::monadic_do;
    use crate::monadic_type_classes::Monad;
    use crate::parser::Parser;

    pub fn fail<A: 'static>() -> Parser<A> {
        Parser::new(|_| None)
    }

    fn space() -> Parser<()> {
        Parser::new(|src| Some(((), src.trim_start())))
    }

    fn item() -> Parser<char> {
        Parser::new(|src| {
            let head = src.chars().next()?;
            let (_, tail) = src.split_at(1);
            Some((head, tail))
        })
    }

    fn text(s: &'static str) -> Parser<&'static str> {
        Parser::new(move |src| {
            let (parsed_s, new_src) = src.split_at_checked(s.len())?;
            (s == parsed_s).then_some((s, new_src))
        })
    }

    pub fn character(c: char) -> Parser<char> {
        monadic_do! {
            space();
            item().satisfy(move |&parsed_c| parsed_c == c)
        }
    }

    pub fn string(s: &'static str) -> Parser<&'static str> {
        monadic_do! {
            space();
            text(s)
        }
    }
}
