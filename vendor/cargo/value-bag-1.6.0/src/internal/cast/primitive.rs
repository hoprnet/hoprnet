/*
This module generates code to try efficiently convert some arbitrary `T: 'static` into
a `Internal`.
*/

#[cfg(feature = "alloc")]
use crate::std::string::String;

use crate::ValueBag;

pub(in crate::internal) fn from_any<'v, T: ?Sized + 'static>(value: &'v T) -> Option<ValueBag<'v>> {
    // NOTE: The casts for unsized values (str) are dubious here. To really do this properly
    // we need https://github.com/rust-lang/rust/issues/81513
    // NOTE: With some kind of const `Any::is<T>` we could do all this at compile-time
    // Older versions of `value-bag` did this, but the infrastructure just wasn't worth
    // the tiny performance improvement
    use crate::std::any::TypeId;

    enum Void {}

    #[repr(transparent)]
    struct VoidRef<'a>(*const &'a Void);

    macro_rules! type_ids {
        ($(
            $(#[cfg($($cfg:tt)*)])*
                $ty:ty,
            )*) => {
            |v: VoidRef<'_>| {
                if TypeId::of::<T>() == TypeId::of::<str>() {
                    // SAFETY: We verify the value is str before casting
                        let v = unsafe { *(v.0 as *const &'_ str) };

                    return Some(ValueBag::from(v));
                }

                    $(
                        $(#[cfg($($cfg)*)])*
                        if TypeId::of::<T>() == TypeId::of::<$ty>() {
                            // SAFETY: We verify the value is $ty before casting
                            let v = unsafe { *(v.0 as *const &'_ $ty) };

                            return Some(ValueBag::from(v));
                        }
                    )*
                    $(
                        $(#[cfg($($cfg)*)])*
                        if TypeId::of::<T>() == TypeId::of::<Option<$ty>>() {
                            // SAFETY: We verify the value is Option<$ty> before casting
                            let v = unsafe { *(v.0 as *const &'_ Option<$ty>) };

                            if let Some(v) = v {
                                return Some(ValueBag::from(v));
                            } else {
                                return Some(ValueBag::from(()));
                            }
                        }
                    )*

                    None
            }
        };
    }

    let type_ids = type_ids![
        usize,
        u8,
        u16,
        u32,
        u64,
        u128,
        isize,
        i8,
        i16,
        i32,
        i64,
        i128,
        f32,
        f64,
        char,
        bool,
        &'static str,
        // We deal with `str` separately because it's unsized
        // str,
        #[cfg(feature = "alloc")]
        String,
    ];

    (type_ids)(VoidRef(&(value) as *const &'v T as *const &'v Void))
}
