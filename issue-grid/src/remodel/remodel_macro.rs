
macro_rules! remodel {
    (
    $(#[$docs:meta])*
    type $remodel:ident remodels ($T:ident) -> ($($Result:tt)*);

    $(

    ( $($in:tt)* ) -> $out:ty $body:block

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
            type Result = $($Result)*;
        }

        $(
        remodel!(@block, $remodel,
            ( $($in)* ) -> $out $body
        );
        )*
    };

    (@block, $remodel:ident,
        ($from:ident : $A:ty) -> $B:ty $body:block
    )=>{
        impl Conv<
            $A,
            $B,
        > for $remodel {
            fn conv($from: $A) -> Self::Result $body
        }
    };

    (@block, $remodel:ident,
        ((
            $from:ident : $A:ty,
            $( $ctx:ident : &$CtxTy:ty ),* $(,)?
        )) -> $B:ty $body:block
    )=>{
        impl<'a> Conv<
            ($A, $(&'a $CtxTy),*),
            $B,
        > for $remodel {
            fn conv(
                tuple: ($A, $(&'a $CtxTy),*)
            ) -> Self::Result {
                let ($from, $( $ctx ),*) = tuple;
                $body
            }
        }
    };
}