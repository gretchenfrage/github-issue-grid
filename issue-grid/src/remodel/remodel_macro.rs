
macro_rules! remodel {
    (
    $(#[$docs:meta])*
    type $remodel:ident remodels ($T:ident) -> ($Result:ident);

    $(
    ($from:ident : $A:ty) -> $B:ty $body:block
    )*
    )=>{
        $(#[$docs])*
        pub enum $remodel {}

        fn conv<A, B>(from: A) -> <$remodel as Remodel<B>>::Result
        where
            $remodel: Conv<A, B> + Remodel<B>
        {
            $remodel::conv(from)
        }

        impl<$T> Remodel<$T> for $remodel {
            type Result = $Result;
        }

        $(
        impl Conv<$A, $B> for $remodel {
            fn conv($from: $A) -> Self::Result $body
        }
        )*
    }
}