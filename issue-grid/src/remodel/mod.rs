
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
    ($Col:ident)=>{
        impl<A, B, C> Conv<$Col<A>, $Col<B>> for C
        where
            C: Conv<A, B>,
            C: Remodel<$Col<B>>,
            <C as Remodel<$Col<B>>>::Result: FromIterator<<C as Remodel<B>>::Result>
        {
            fn conv(from: $Col<A>) -> Self::Result {
                from.into_iter()
                    .map(C::conv)
                    .collect()
            }
        }
    };
}

conv_collection!(Vec);
conv_collection!(Option);
