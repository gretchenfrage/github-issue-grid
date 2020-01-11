
#![allow(unused_macros)]

macro_rules! serde_as_list {
    (
    struct $struct:ident;
    $($t:tt)*
    )=>{
        impl serde::Serialize for $struct {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeSeq;

                let mut s_seq = s.serialize_seq(None)?;

                serde_as_list!(@ser, $struct, s_seq, self, ($($t)*))
            }
        }

        impl<'de> serde::Deserialize<'de> for $struct {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>
            {
                use serde::de::{Visitor, SeqAccess, Error};
                use std::fmt::{self, Formatter};

                struct V;
                impl<'de2> Visitor<'de2> for V {
                    type Value = $struct;

                    fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                        f.write_str(concat!(
                            "sequence form of ",
                            stringify!($struct)
                        ))
                    }

                    fn visit_seq<A>(self, mut d_seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: SeqAccess<'de2>
                    {
                        serde_as_list!(@de, $struct, d_seq, self, ($struct {}), ($($t)*))
                    }
                }

                d.deserialize_seq(V)
            }
        }
    };

    // ====

    // ser field case
    (
    @ser, $struct:ty, $s_seq:expr, $self:expr,
    (
    field $field:ident;
    $($t:tt)*
    )
    )=>{{
        $s_seq.serialize_element(&$self.$field)?;

        // recurse
        serde_as_list!(@ser, $struct, $s_seq, $self, ($($t)*))
    }};

    // ser option_tail case
    (
    @ser, $struct:ty, $s_seq:expr, $self:expr,
    (
    option_tail $field:ident;
    $($t:tt)*
    )
    )=>{{
        if let Some(ref vec) = $self.$field {
            for elem in vec {
                $s_seq.serialize_element(elem)?;
            }
        }

        // recurse into base case
        serde_as_list!(@assert_empty_parens, ($($t)*));
        serde_as_list!(@ser, $struct, $s_seq, $self, ($($t)*))
    }};

    // ser base case
    (
    @ser, $struct:ty, $s_seq:expr, $self:expr,
    ()
    )=>{{
        $s_seq.end()
    }};

    // ====

    // de field case
    (
    @de, $struct:ty, $d_seq:expr, $self:expr,
    (   // constructor accumulator
        $struct_cons:ident {
            $($t_cons:tt)*
        }
    ),
    (
    field $field:ident;
    $($t:tt)*
    )
    )=>{{
        let $field = $d_seq.next_element()?
            .ok_or_else(|| A::Error::custom(concat!(
                stringify!($struct),
                ".",
                stringify!($field),
            )))?;

        // recurse
        serde_as_list!(
            @de, $struct, $d_seq, $self,
            (
                $struct_cons {
                    $($t_cons)*
                    $field: $field,
                }
            ),
            ($($t)*)
        )
    }};

    // de option_tail case
    (
    @de, $struct:ty, $d_seq:expr, $self:expr,
    (   // constructor accumulator
        $struct_cons:ident {
            $($t_cons:tt)*
        }
    ),
    (
    option_tail $field:ident;
    $($t:tt)*
    )
    )=>{{
        let mut tail = Vec::new();
        while let Some(elem) = $d_seq.next_element()? {
            tail.push(elem);
        }
        let $field = match tail.len() {
            0 => None,
            _ => Some(tail),
        };

        // recurse
        serde_as_list!(@assert_empty_parens, ($($t)*));
        serde_as_list!(
            @de, $struct, $d_seq, $self,
            (
                $struct_cons {
                    $($t_cons)*
                    $field: $field,
                }
            ),
            ($($t:tt)*)
        )
    }};

    // de base case
    (
    @de, $struct:ty, $d_seq:expr, $self:expr,
    (   // constructor accumulator
        $struct_cons:ident {
            $($t_cons:tt)*
        }
    ),
    ()
    )=>{{
        Ok($struct_cons {
            $($t_cons)*
        })
    }};

    // ====

    (@assert_empty_parens, ())=>{};
    (@deform, $($t:tt)*)=>{
        $($t)*
    };

}