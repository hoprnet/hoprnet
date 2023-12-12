//! Implementations of OpenAPI `oneOf` and `anyOf` types, assuming rules are just types
#[cfg(feature = "conversion")]
use frunk_enum_derive::LabelledGenericEnum;
use serde::{
    de::Error,
    Deserialize, Deserializer, Serialize, Serializer,
    __private::de::{Content, ContentRefDeserializer},
};
use std::str::FromStr;
use std::string::ToString;

// Define a macro to define the common parts between `OneOf` and `AnyOf` enums for a specific
// number of inner types.
macro_rules! common_one_any_of {
    (
        $schema:ident,
        $t:ident,
        $($i:ident),*
    ) => {
        #[doc = concat!("`", stringify!($t), "` type.\n\nThis allows modelling of ", stringify!($schema), " JSON schemas.")]
        #[cfg_attr(feature = "conversion", derive(LabelledGenericEnum))]
        #[derive(Debug, PartialEq, Clone)]
        pub enum $t<$($i),*> where
            $($i: PartialEq,)*
        {
            $(
                #[doc = concat!("`", stringify!($i), "` variant of `", stringify!($t), "`")]
                $i($i)
            ),*
        }

        impl<$($i),*> Serialize for $t<$($i),*> where
            $($i: PartialEq + Serialize,)*
        {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$i(inner) => inner.serialize(serializer)),*
                }
            }
        }

        impl<$($i),*> ToString for $t<$($i),*> where
            $($i: PartialEq + ToString,)*
        {
            fn to_string(&self) -> String {
                match self {
                    $(Self::$i(inner) => inner.to_string()),*
                }
            }
        }
    }
}

// Define a macro to define the `OneOf` enum for a specific number of inner types.
macro_rules! one_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        common_one_any_of!(oneOf, $t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                let content = Content::deserialize(deserializer)?;
                let mut result = Err(De::Error::custom("data did not match any within oneOf"));
                $(
                    if let Ok(inner) = $i::deserialize(ContentRefDeserializer::<De::Error>::new(&content)) {
                        if result.is_err() {
                            result = Ok(Self::$i(inner));
                        } else {
                            return Err(De::Error::custom("data matched multiple within oneOf"))
                        }
                    }
                )*
                result
            }
        }

        impl<$($i),*> FromStr for $t<$($i),*> where
            $($i: PartialEq + FromStr,)*
        {
            type Err = &'static str;
            fn from_str(x: &str) -> Result<Self, Self::Err> {
                let mut result = Err("data did not match any within oneOf");
                $(
                    if let Ok(inner) = $i::from_str(x) {
                        if result.is_err() {
                            result = Ok(Self::$i(inner));
                        } else {
                            return Err("data matched multiple within oneOf")
                        }
                    }
                )*
                result
            }
        }
    }
}

// Use the `one_of!` macro to define the `OneOf` enum for 1-16 inner types.
one_of!(OneOf1, A);
one_of!(OneOf2, A, B);
one_of!(OneOf3, A, B, C);
one_of!(OneOf4, A, B, C, D);
one_of!(OneOf5, A, B, C, D, E);
one_of!(OneOf6, A, B, C, D, E, F);
one_of!(OneOf7, A, B, C, D, E, F, G);
one_of!(OneOf8, A, B, C, D, E, F, G, H);
one_of!(OneOf9, A, B, C, D, E, F, G, H, I);
one_of!(OneOf10, A, B, C, D, E, F, G, H, I, J);
one_of!(OneOf11, A, B, C, D, E, F, G, H, I, J, K);
one_of!(OneOf12, A, B, C, D, E, F, G, H, I, J, K, L);
one_of!(OneOf13, A, B, C, D, E, F, G, H, I, J, K, L, M);
one_of!(OneOf14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
one_of!(OneOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
one_of!(OneOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// Define a macro to define the `AnyOf` enum for a specific number of inner types.
macro_rules! any_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        common_one_any_of!(anyOf, $t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                let content = Content::deserialize(deserializer)?;
                $(
                    if let Ok(inner) = $i::deserialize(ContentRefDeserializer::<De::Error>::new(&content)) {
                        return Ok(Self::$i(inner));
                    }
                )*
                Err(De::Error::custom("data did not match any within anyOf"))
            }
        }

        impl<$($i),*> FromStr for $t<$($i),*> where
            $($i: PartialEq + FromStr,)*
        {
            type Err = &'static str;
            fn from_str(x: &str) -> Result<Self, Self::Err> {
                $(
                    if let Ok(inner) = $i::from_str(x) {
                        return Ok(Self::$i(inner));
                    }
                )*
                Err("data did not match any within anyOf")
            }
        }
    }
}

// Use the `any_of!` macro to define the `AnyOf` enum for 1-16 inner types.
any_of!(AnyOf1, A);
any_of!(AnyOf2, A, B);
any_of!(AnyOf3, A, B, C);
any_of!(AnyOf4, A, B, C, D);
any_of!(AnyOf5, A, B, C, D, E);
any_of!(AnyOf6, A, B, C, D, E, F);
any_of!(AnyOf7, A, B, C, D, E, F, G);
any_of!(AnyOf8, A, B, C, D, E, F, G, H);
any_of!(AnyOf9, A, B, C, D, E, F, G, H, I);
any_of!(AnyOf10, A, B, C, D, E, F, G, H, I, J);
any_of!(AnyOf11, A, B, C, D, E, F, G, H, I, J, K);
any_of!(AnyOf12, A, B, C, D, E, F, G, H, I, J, K, L);
any_of!(AnyOf13, A, B, C, D, E, F, G, H, I, J, K, L, M);
any_of!(AnyOf14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
any_of!(AnyOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
any_of!(AnyOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
