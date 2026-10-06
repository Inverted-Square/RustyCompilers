#![allow(unused)]

use crate::monadic_do;
use crate::monadic_type_classes::{Alternative, Applicative, Functor, Monad};
use family_member_derive::FamilyMember;
use std::ops::{BitOr, Shr};

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
            if cond(&t) { Parser::ret(t) } else { Parser::empty() }
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

        fn apply<B: 'static, F>(self, mf: Parser<F>) -> Parser<B>
        where
            F: FnOnce(A) -> B + 'static,
        {
            Parser::new(|src| {
                mf.parse(src)
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

        fn alternative(self, ma: Self) -> Self {
            Parser::new(|src| self.parse(src).or_else(|| ma.parse(src)))
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
            Err(_) => Parser::empty(),
        }
    }
}

pub mod tam_grammar {
    pub enum Program {
        LetIn(Vec<Declaration>, Command),
    }

    pub enum Declaration {
        VarDeclare(String, Type),
        VarInit(String, Type, Expr),
        Func(String, Vec<FuncParam>, Type, Expr),
    }

    pub enum Command {
        VarAssign(String, Expr),
        IfThenElse(Expr, Box<Command>, Box<Command>),
        While(Expr, Box<Command>),
        GetInt(i32),
        PrintInt(Expr),
        BeginEnd(Vec<Command>),
    }

    pub enum Type {
        Int,
        Bool,
    }

    #[derive(Debug)]
    pub enum Expr {
        LitNat(u32),
        LitBool(bool),
        Var(String),
        BinOp(BinOperator, Box<Expr>, Box<Expr>),
        UnOp(UnOperator, Box<Expr>),
        Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
        FuncApp(String, Vec<Expr>),
    }

    pub enum FuncParam {
        Param(String, Type),
    }

    #[derive(Debug)]
    pub enum BinOperator {
        Add,
        Sub,
        Mul,
        Div,
        LogAnd,
        LogOr,
        LT,
        LE,
        GT,
        GE,
        EQ,
        NE,
    }

    #[derive(Debug)]
    pub enum UnOperator {
        Minus,
        LogNot,
    }
}

pub mod parsers {
    use crate::monadic_do;
    use crate::monadic_type_classes::Monad;
    use crate::parser::Parser;
    use crate::parser::tam_grammar::BinOperator::LogOr;
    use crate::parser::tam_grammar::Expr::{BinOp, Conditional, LitNat};
    use crate::parser::tam_grammar::{Expr, Type};

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

    pub fn string_satisfy<F>(f: F) -> Parser<String>
    where
        F: Fn(&char) -> bool + Clone + 'static,
    {
        monadic_do! {
            c <- item().satisfy(f.clone());
            rest <- string_satisfy(f) | Parser::ret(String::new());
            return format!("{c}{rest}");
        }
    }

    pub fn identifier() -> Parser<String> {
        monadic_do! {
            c <- item().satisfy(|&c| c.is_ascii_alphabetic());
            rest <- string_satisfy(|&c| c.is_ascii_alphanumeric()) | Parser::ret(String::new());
            return format!("{c}{rest}");
        }
    }

    pub fn tam_type() -> Parser<Type> {
        let parse_type_int = monadic_do! {
            string("int");
            return Type::Int;
        };
        let parse_type_bool = monadic_do! {
            string("bool");
            return Type::Bool;
        };
        parse_type_int | parse_type_bool
    }

    pub fn expr() -> Parser<Expr> {
        let parse_conditional = monadic_do! {
            expr1 <- log_or();
            character('?');
            expr2 <- expr();
            character(':');
            expr3 <- expr();
            return Conditional(Box::new(expr1), Box::new(expr2), Box::new(expr3));
        };
        parse_conditional | log_or()
    }

    pub fn log_or() -> Parser<Expr> {
        monadic_do! {
            expr1 <- nat();
            exprs <- many_log_or();
            return exprs.into_iter().rfold(LitNat(expr1), |final_e, e| BinOp(LogOr, Box::new(final_e), Box::new(e)));
        }
    }

    fn many_log_or() -> Parser<Vec<Expr>> {
        let many_expr = monadic_do! {
            expr <- string("||").sequence(nat());
            mut exprs <- many_log_or();
            let () = exprs.push(LitNat(expr));
            return exprs;
        };
        many_expr | Parser::ret(vec![])
    }

    pub fn nat() -> Parser<u32> {
        monadic_do! {
            num_str <- string_satisfy(|&c| c.is_ascii_digit()).token();
            num_str.parse().into()
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
