//! This module contains various helpful macros which are not
//! strictly part of Hexadecimal serialization/deserialization.

/// Implement common conversion traits for the newtype pattern.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype {
    ($outer:ident, $inner: ty) => {
        impl From<$inner> for $outer {
            fn from(inner: $inner) -> Self {
                $outer(inner)
            }
        }

        impl<R: ?Sized> AsRef<R> for $outer
        where
            $inner: AsRef<R>,
        {
            fn as_ref(&self) -> &R {
                self.0.as_ref()
            }
        }

        impl<R: ?Sized> AsMut<R> for $outer
        where
            $inner: AsMut<R>,
        {
            fn as_mut(&mut self) -> &mut R {
                self.0.as_mut()
            }
        }

        impl ::std::ops::Deref for $outer {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for $outer {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl ::std::borrow::Borrow<$inner> for $outer {
            fn borrow(&self) -> &$inner {
                &self.0
            }
        }

        impl ::std::borrow::BorrowMut<$inner> for $outer {
            fn borrow_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }
    };
}

/// implements useful traits for the 'newtype' pattern.
/// this macro is automatically implemented by `impl_newtype_bytearray`,
/// so prefer that macro if `inner` is a byte-array (`[u8;n]`).
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_old {
    ($outer: ident, $inner: ty) => {
        // dereference to inner value.
        impl ::std::ops::Deref for $outer {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // dereference to inner value.
        impl ::std::ops::DerefMut for $outer {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        // convert from the inner value to the outer value.
        impl ::std::convert::From<$inner> for $outer {
            fn from(inner: $inner) -> Self {
                $outer(inner)
            }
        }

        // get immutable reference to inner value.
        impl AsRef<$inner> for $outer {
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        // get mutable reference to inner value.
        impl AsMut<$inner> for $outer {
            fn as_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }
    };
}

/// implements useful traits for array newtypes
/// (e.g.; `Foo([Bar;n])`).  Includes all implementations from
/// the `impl_newtype` macro.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_array {
    ($outer: ident, $inner: ty, $len: expr) => {
        impl_newtype!($outer, [u8; $len]);
        /*
                // get reference as byte-slice.
                impl AsRef<[$inner]> for $outer {
                    fn as_ref(&self) -> &[$inner] {
                        self.0.as_ref()
                    }
                }

                impl AsMut<[$inner]> for $outer {
                    fn as_mut(&mut self) -> &mut [$inner] {
                        self.0.as_mut()
                    }
                }
        */
    };
}

/// Apply the `LowerHex` and `UpperHex` traits.  TODO: this macro doesn't
/// generalize properly at the moment.  Make it not terrible plz.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_hexfmt {
    ($outer: ident, $lowtoken: expr, $uptoken: expr) => {
        // implement the `LowerHex` trait to allow generation
        // of lowercase hexadecimal representations.
        impl ::std::fmt::LowerHex for $outer {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let bytes: &[u8] = self.as_ref();
                for val in bytes.iter() {
                    write!(f, $lowtoken, val)?;
                }
                Ok(())
            }
        }

        // implement the `UpperHex` trait to allow generation
        // of uppercase hexadecimal representations.
        impl ::std::fmt::UpperHex for $outer {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let bytes: &[u8] = self.as_ref();
                for val in bytes.iter() {
                    write!(f, $uptoken, val)?;
                }
                Ok(())
            }
        }
    };
}

/// Implement useful traits for byte-array newtypes
/// (e.g.; `Foo([u8;n])`).  includes implementations
/// from `impl_newtype_array`.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_bytearray {
    ($outer: ident, $len: expr) => {
        impl_newtype_array!($outer, u8, $len);
        impl_newtype_hexfmt!($outer, "{:02x}", "{:02X}");
    };
}

/// implements useful traits for array newtypes
/// (e.g.; `Foo([Bar;n])`) if grater than 32 elements in length.
/// Includes all implementations from the `impl_newtype` macro,
/// as well as a number of useful traits which cannot be
/// derived via `#[derive(...)]`.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_array_ext {
    ($outer: ident, $inner: ty, $len:expr) => {
        // implement everything from the nomral bytearray macro.
        impl_newtype_array!($outer, $inner, $len);
        /*
                // manually implemented `Debug` trait for printouts.
                impl ::std::fmt::Debug for $outer {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        let s: &[$inner] = self.as_ref();
                        write!(f, "{}({:?})",stringify!($ident),s)
                    }
                }
        */
        // manually implement `PartialEq` for comparison operations.
        impl ::std::cmp::PartialEq for $outer {
            fn eq(&self, other: &$outer) -> bool {
                let sref: &[$inner] = self.as_ref();
                let oref: &[$inner] = other.as_ref();
                sref == oref
            }
        }

        // manually flag type as `Eq` for full equivalence relations.
        impl ::std::cmp::Eq for $outer {}
    };
}

/// implements additional useful traits for numeric-array newtypes
/// (e.g.; `Foo([usize;n])`) of greater than 32 elements.
/// Includes all impls from `impl_newtype_numarray`.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_numarray_ext {
    ($outer: ident, $inner: ty, $len:expr) => {
        // manually implemented `Clone` trait for easy copying.
        impl Clone for $outer {
            fn clone(&self) -> Self {
                let mut buf = [<$inner as Default>::default(); $len];
                let s: &[$inner] = self.as_ref();
                for (idx, itm) in s.iter().enumerate() {
                    buf[idx] = *itm;
                }
                buf.into()
            }
        }

        // manuall implement `Default` trait for getting empty instances.
        impl Default for $outer {
            fn default() -> Self {
                $outer([<$inner as Default>::default(); $len])
            }
        }
    };
}

/// implements useful traits for byte-array newtypes
/// (e.g.; `Foo([u8;n])`) for arrays of greater than 32 elements.
/// Includes all implementations the `impl_newtype_array_ext`
/// and `impl_newtype_numarray_ext` macros.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_newtype_bytearray_ext {
    ($outer: ident, $len:expr) => {
        impl_newtype_array_ext!($outer, u8, $len);
        impl_newtype_hexfmt!($outer, "{:02x}", "{:02X}");
        impl_newtype_numarray_ext!($outer, u8, $len);
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn implementation() {
        struct Bar([u8; 36]);
        impl_newtype_bytearray_ext!(Bar, 36);
    }
}
