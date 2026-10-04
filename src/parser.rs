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

    pub fn token(self) -> Self {
        monadic_do! {
            parsers::space();
            self
        }
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

impl<T: 'static, E> From<Result<T, E>> for Parser<T> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(val) => Parser::ret(val),
            Err(_) => parsers::fail(),
        }
    }
}

pub mod parsers {
    use crate::monadic_do;
    use crate::monadic_type_classes::Monad;
    use crate::parser::Parser;

    pub fn fail<A: 'static>() -> Parser<A> {
        Parser::new(|_| None)
    }

    pub fn space() -> Parser<()> {
        Parser::new(|src| Some(((), src.trim_start())))
    }

    pub fn item() -> Parser<char> {
        Parser::new(|src| {
            let head = src.chars().next()?;
            let (_, tail) = src.split_at(1);
            Some((head, tail))
        })
    }

    pub fn character(c: char) -> Parser<char> {
        item().satisfy(move |&parsed_c| parsed_c == c).token()
    }

    pub fn string(s: &'static str) -> Parser<&'static str> {
        Parser::new(move |src| {
            let (parsed_s, new_src) = src.split_at_checked(s.len())?;
            (s == parsed_s).then_some((s, new_src))
        })
        .token()
    }

    pub fn nat() -> Parser<u32> {
        monadic_do! {
            num_str <- nat_str().token();
            num_str.parse().into()
        }
    }

    fn nat_str() -> Parser<String> {
        monadic_do! {
            digit <- item().satisfy(|&c| "0123456789".contains(c));
            digits <- nat_str() | Parser::ret(String::new());
            return format!("{digit}{digits}");
        }
    }

    pub fn boolean() -> Parser<bool> {
        let parse_true = monadic_do! {
            string("true");
            return true;
        };
        let parse_false = monadic_do! {
            string("false");
            return false;
        };
        parse_true | parse_false
    }
}
