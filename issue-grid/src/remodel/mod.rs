
use crate::sort::PatternList;
use std::iter::FromIterator;

#[macro_use]
pub mod remodel_macro;

/// Re-modelling our config file.
pub mod config;

/// Re-modelling github data.
pub mod github;

/// Uninhabited type which represents a system of converting
/// from one data model to another.
///
/// This particular trait represents a type constructor, so
/// that this system can be generic over fallible and
/// infalliable conversion system.
pub trait Remodel<B> {
    type Result;
}

pub trait Conv<A, B>
where
    Self: Remodel<B>
{
    fn conv(from: A) -> Self::Result;
}

// == conversion implementations elevate to collections ==

macro_rules! conv_collection {
    ($From:ident -> $To:ident)=>{
        impl<A, B, C> Conv<$From<A>, $To<B>> for C
        where
            C: Conv<A, B>,
            C: Remodel<$To<B>>,
            <C as Remodel<$To<B>>>::Result: FromIterator<<C as Remodel<B>>::Result>
        {
            fn conv(from: $From<A>) -> Self::Result {
                from.into_iter()
                    .map(C::conv)
                    .collect()
            }
        }
    };
    ($Col:ident)=>{ conv_collection! { $Col -> $Col } };
}

conv_collection!(Vec);
conv_collection!(Vec -> PatternList);

pub trait FromOption<T>: Sized {
    fn from_option(from: Option<T>) -> Self;
}

impl<T> FromOption<T> for Option<T> {
    fn from_option(from: Option<T>) -> Self { from }
}

impl<I, E> FromOption<Result<I, E>> for Result<Option<I>, E> {
    fn from_option(from: Option<Result<I, E>>) -> Self { from.transpose() }
}

impl<A, B, C> Conv<Option<A>, Option<B>> for C
where
    C: Conv<A, B>,
    C: Remodel<Option<B>, Result=Option<B>>,
    <C as Remodel<Option<B>>>::Result: FromOption<<C as Remodel<B>>::Result>
{
    fn conv(from: Option<A>) -> Self::Result {
        Self::Result::from_option(from.map(C::conv))
    }
}
