// {{{ Documentation
//! Simple checked newtype generator, primarily for use with [serde](https://serde.rs).
//! Serde support (and dependency) may be disabled with `default_features = false`.
//! This is `#![no_std]` library.
//!
//! Usage:
//! ```
//! # use validated_newtype::validated_newtype;
//! # use serde_json;
//! validated_newtype! {
//!     /// Documentation comments and attributes are applied
//!     #[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
//!     // base type name => optional visibility + newtype name
//!     u32 => pub Percent
//!     // condition to check when creating/deserializing
//!     if |n: &u32| *n <= 100;
//!     // error message if condition is not met
//!     error "percent must be in range 0-100"
//! }
//!
//! let x: Percent = serde_json::from_str("42").unwrap();
//! assert_eq!(*x, 42);
//! let y: Result<Percent, _> = serde_json::from_str("1337");
//! assert!(y.is_err());
//! ```
//! Instances of generated newtype can be created only via [TryFrom] or [Deserialize],
//! so they always hold valid data.
//!
//! ## Dynamic error generation
//! ```
//! # use core::convert::TryInto;
//! # use validated_newtype::validated_newtype;
//! # use serde_json;
//! validated_newtype! {
//!     #[derive(Debug)]
//!     u32 => pub Percent
//!     if |n: &u32| *n <= 100;
//!     else |n: &u32| format!("number {} is not in range 0-100", n) => String
//! }
//!
//! // Deserialize for newtypes uses try_into internally
//! let x: Result<Percent, _> = 1337.try_into();
//! assert!(x.is_err());
//! assert_eq!(x.unwrap_err(), "number 1337 is not in range 0-100");
//! ```
//! ## Manually implement [TryFrom]
//! ```
//! # use core::convert::TryFrom;
//! # use validated_newtype::validated_newtype;
//! # use serde_json;
//! validated_newtype! {
//!     #[derive(Debug)]
//!     u32 => pub Percent
//! }
//!
//! impl TryFrom<u32> for Percent {
//!     type Error = &'static str;
//!
//!     fn try_from(val: u32) -> Result<Self, Self::Error> {
//!         if val > 100 {
//!             Err("percent must be in range 0-100")
//!         } else {
//!             Ok(Self(val))
//!         }
//!     }
//! }
//!
//! let x: Percent = serde_json::from_str("42").unwrap();
//! assert_eq!(*x, 42);
//! let y: Result<Percent, _> = serde_json::from_str("1337");
//! assert!(y.is_err());
//! ```
//!
//! [TryFrom]: https://doc.rust-lang.org/stable/core/convert/trait.TryFrom.html
//! [Deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
// }}}

#![no_std]

#[cfg(feature = "serde")]
#[doc(hidden)]
#[macro_export]
macro_rules! add_deserialize {
    ($type:ident, $parent:ty) => {
        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                use core::convert::TryInto as _;
                use serde::de::Error as _;
                <$parent as serde::Deserialize>::deserialize(deserializer)?
                    .try_into()
                    .map_err(D::Error::custom)
            }
        }
    };
}

#[cfg(not(feature = "serde"))]
#[doc(hidden)]
#[macro_export]
macro_rules! add_deserialize {
    ($type:ident, $parent:ty) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! add_try_from {
    ($type:ident, $parent:ty, $predicate:expr, $error_type:ty, $error:expr) => {
        impl core::convert::TryFrom<$parent> for $type {
            type Error = $error_type;

            fn try_from(val: $parent) -> Result<Self, $error_type> {
                if $predicate(&val) {
                    Ok($type(val))
                } else {
                    Err($error(&val).into())
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! add_deref {
    ($type:ident, $parent:ty) => {
        impl core::ops::Deref for $type {
            type Target = $parent;

            fn deref(&self) -> &$parent {
                &self.0
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! uniform_validated_newtype {
    (
        $( #[$attr:meta] )*
        $parent:ty => $vis:vis $type:ident
        $(
            if $predicate:expr;
            else $error:expr => $error_type:ty
        )?
    ) => {
        #[allow(unused_attributes)]
        $( #[$attr] )*
        $vis struct $type($parent);
        $(
            $crate::add_try_from!($type, $parent, $predicate, $error_type, $error);
        )?
        $crate::add_deserialize!($type, $parent);
        $crate::add_deref!($type, $parent);
    }
}

/// Macro to create deserializable newtype with predicate validation.
/// See crate docs for examples.
#[macro_export]
macro_rules! validated_newtype {
    (
        $( #[$attr:meta] )*
        $parent:ty => $vis:vis $type:ident
        $(
            if $predicate:expr;
            else $error:expr => $error_type:ty
        )?
    ) => {
        $crate::uniform_validated_newtype! {
            $( #[$attr] )*
            $parent => $vis $type
            $(
                if $predicate;
                else $error => $error_type
            )?
        }
    };
    (
        $( #[$attr:meta] )*
        $parent:ty => $vis:vis $type:ident
        if $predicate:expr;
        error $message:literal
    ) => {
        $crate::uniform_validated_newtype! {
            $( #[$attr] )*
            $parent => $vis $type
            if $predicate;
            else |_| $message => &'static str
        }
    };
}
