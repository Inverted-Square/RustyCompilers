#![allow(unused)]

pub trait Family {
    type Member<T>;
}

pub trait FamilyMember<T> {
    type Of: Family<Member<T> = Self>;
}

pub type SelfOfT<T, U> = <<T as FamilyMember<U>>::Of as Family>::Member<U>;
pub type SelfOfTU<T, U, V> = <<T as FamilyMember<U>>::Of as Family>::Member<V>;

pub trait Functor<A>: FamilyMember<A> {
    fn fmap<B, F>(self, f: F) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> B + 'static;
}

pub trait Applicative<A>: Functor<A> {
    fn pure(a: A) -> SelfOfT<Self, A>;
    fn apply<B, F>(self, pf: SelfOfTU<Self, A, F>) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> B + 'static;
}

pub trait Monad<A>: Applicative<A> {
    fn ret(a: A) -> SelfOfT<Self, A>;
    fn bind<B, F>(self, f: F) -> SelfOfTU<Self, A, B>
    where
        F: FnOnce(A) -> SelfOfTU<Self, A, B> + 'static;
}

pub trait Alternative<A>: Applicative<A> {
    fn empty() -> SelfOfT<Self, A>;
    fn alternative(self, pa: SelfOfT<Self, A>) -> SelfOfT<Self, A>;
}
