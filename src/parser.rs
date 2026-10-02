#![allow(unused)]

use crate::monadic_type_classes::Alternative;
use crate::parser::Parser::P;
use crate::parser::byte_stream::ByteStream;
use std::ops::BitOr;

mod byte_stream {
    use std::ops::Deref;

    #[derive(Clone)]
    pub struct ByteStream(Vec<u8>);

    impl Deref for ByteStream {
        type Target = Vec<u8>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<&[u8]> for ByteStream {
        fn from(value: &[u8]) -> Self {
            Self(value.to_vec())
        }
    }

    impl From<String> for ByteStream {
        fn from(value: String) -> Self {
            value.as_bytes().into()
        }
    }
}

type Matcher<T> = dyn FnOnce(ByteStream) -> Option<(T, ByteStream)> + 'static;

pub enum Parser<T> {
    P(Box<Matcher<T>>),
}

impl<T> Parser<T> {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce(ByteStream) -> Option<(T, ByteStream)> + 'static,
    {
        P(Box::new(f))
    }

    pub fn parse_str(self, src: String) -> Option<(T, String)> {
        let P(p) = self;
        let (v, new_src) = p(src.into())?;
        let new_src = String::from_utf8((*new_src).clone()).ok()?;
        Some((v, new_src))
    }

    fn parse(self, src: ByteStream) -> Option<(T, ByteStream)> {
        let P(p) = self;
        p(src)
    }
}

mod monadic_parser {
    use crate::monadic_type_classes::{
        Alternative, Applicative, Family, FamilyMember, Functor, Monad,
    };
    use crate::parser::Parser;

    pub struct ParserFamily;

    impl Family for ParserFamily {
        type Member<T> = Parser<T>;
    }

    impl<T> FamilyMember<T> for Parser<T> {
        type Of = ParserFamily;
    }

    impl<A: 'static> Functor<A> for Parser<A> {
        fn fmap<B, F>(self, f: F) -> Parser<B>
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

        fn apply<B, F>(self, pf: Parser<F>) -> Parser<B>
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
        fn ret(a: A) -> Self {
            Self::pure(a)
        }

        fn bind<B, F>(self, f: F) -> Parser<B>
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
            Parser::new(|src| self.parse(src.clone()).or_else(|| pa.parse(src)))
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
    use crate::monadic_type_classes::Functor;
    use crate::parser::Parser;

    pub fn item() -> Parser<u8> {
        Parser::new(|src| match src.as_slice() {
            [] => None,
            [head, tail @ ..] => Some((*head, tail.into())),
        })
    }

    pub fn item_char() -> Parser<char> {
        item().fmap(|b| b as char)
    }
}
