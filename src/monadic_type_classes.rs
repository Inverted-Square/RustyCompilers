#![allow(unused)]

pub trait Family {
    type Member<T>;
}

pub trait FamilyMember<T> {
    type Of: Family<Member<T> = Self>;
}

pub type SelfOfTU<T, U, V> = <<T as FamilyMember<U>>::Of as Family>::Member<V>;

pub trait Functor<A>: FamilyMember<A> {
    fn fmap<B, F>(self, f: F) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> B + 'static;
}

pub trait Applicative<A>: Functor<A> {
    fn pure(a: A) -> Self;
    fn apply<B, F>(self, pf: SelfOfTU<Self, A, F>) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> B + 'static;
}

pub trait Monad<A>: Applicative<A> {
    fn ret(a: A) -> Self
    where
        Self: Sized,
    {
        Self::pure(a)
    }

    fn bind<B, F>(self, f: F) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> SelfOfTU<Self, A, B> + 'static;
}

pub trait Alternative<A>: Applicative<A> {
    fn empty() -> Self;
    fn alternative(self, pa: Self) -> Self;
}

#[macro_export]
macro_rules! monadic_do {
    (return $e:expr;) => {
        $crate::monadic_type_classes::Monad::ret($e)
    };

    (let $p:pat = $e:expr;$($r:tt)*) => {{
        let $p = $e;
        monadic_do!($($r)*)
    }};

    (_ <- $e:expr;$($r:tt)*) => {
        $e.bind(move |_| { monadic_do!($($r)*) })
    };

    ($binding:ident <- $e:expr;$($r:tt)*) => {
        $e.bind(move |$binding| { monadic_do!($($r)*) })
    };

    ($e:expr;$($r:tt)*) => {
        $e.bind(move |_| { monadic_do!($($r)*) })
    };

    ($e:expr) => {
        $e
    };
}
