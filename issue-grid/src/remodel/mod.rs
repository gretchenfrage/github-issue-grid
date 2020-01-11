
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
conv_collection!(Option);
conv_collection!(Vec -> PatternList);