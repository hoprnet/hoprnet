//! Extensions for the built-in floating point types.

/// Set of methods to safely convert floating number into an integer.
///
/// Currently, the main way to do so is to use [`as`][as_convert] conversion.
/// However, such an approach may not be suitable if saturating conversion is
/// not desired.
///
/// However, saturating conversion is also provided as an expicit alternative
/// to `as` conversion (e.g. to avoid warnings when [`clippy::as_conversions`][clippy_as]
/// lint is enabled).
///
/// [as_convert]: https://doc.rust-lang.org/nomicon/casts.html
/// [clippy_as]: https://rust-lang.github.io/rust-clippy/master/index.html#as_conversions
///
/// ## Implementors
///
/// This trait is implemented for both [`f32`] and [`f64`].
pub trait FloatConvert<Int>: Sized {
    /// Floors the floating number and attempts to convert it into an integer.
    /// See [`f32::floor`] for description of flooring logic.
    ///
    /// Returns `None` if the value will not fit into the integer range or value
    /// is not a number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use stdext::prelude::FloatConvert;
    ///
    /// let valid: Option<u8> = 10.5f32.checked_floor();
    /// let too_big: Option<u8> = 256f32.checked_floor();
    /// let nan: Option<u8> = f32::NAN.checked_floor();
    ///
    /// assert_eq!(valid, Some(10u8));
    /// assert_eq!(too_big, None);
    /// assert_eq!(nan, None);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_floor(self) -> Option<Int>;

    /// Ceils the floating number and attempts to convert it into an integer.
    /// See [`f32::ceil`] for description of ceiling logic.
    ///
    /// Returns `None` if the value will not fit into the integer range or value
    /// is not a number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use stdext::prelude::FloatConvert;
    ///
    /// let valid: Option<u8> = 10.5f32.checked_ceil();
    /// let too_big: Option<u8> = 256f32.checked_ceil();
    /// let nan: Option<u8> = f32::NAN.checked_ceil();
    ///
    /// assert_eq!(valid, Some(11u8));
    /// assert_eq!(too_big, None);
    /// assert_eq!(nan, None);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_ceil(self) -> Option<Int>;

    /// Rounds the floating number and attempts to convert it into an integer.
    /// See [`f32::round`] for description of rounding logic.
    ///
    /// Returns `None` if the value will not fit into the integer range or value
    /// is not a number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use stdext::prelude::FloatConvert;
    ///
    /// let valid: Option<u8> = 10.51f32.checked_round(); // Will be rounded up.
    /// let too_big: Option<u8> = 256f32.checked_round();
    /// let nan: Option<u8> = f32::NAN.checked_round();
    ///
    /// assert_eq!(valid, Some(11u8));
    /// assert_eq!(too_big, None);
    /// assert_eq!(nan, None);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_round(self) -> Option<Int>;

    /// Behaves the same as `number.floor() as <type>`.
    /// See [`f32::floor`] for description of flooring logic.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturated_floor(self) -> Int;

    /// Behaves the same as `number.ceil() as <type>`.
    /// See [`f32::ceil`] for description of flooring logic.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturated_ceil(self) -> Int;

    /// Behaves the same as `number.round() as <type>`.
    /// See [`f32::round`] for description of flooring logic.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturated_round(self) -> Int;
}

macro_rules! checked_impl {
    ($val:ident.$fn:ident(), $int:ty) => {{
        if $val.is_nan() || $val.is_infinite() {
            return None;
        }
        let converted = $val.$fn();
        if <$int>::MIN as Self <= converted && converted <= <$int>::MAX as Self {
            Some(converted as $int)
        } else {
            None
        }
    }};
}

macro_rules! saturated_impl {
    ($val:ident.$fn:ident(), $int:ty) => {{
        $val.$fn() as $int
    }};
}

macro_rules! impl_float_convert {
    ($float:ty, $($int:ty),+) => {
        $(impl FloatConvert<$int> for $float {
            fn checked_floor(self) -> Option<$int> {
                checked_impl!(self.floor(), $int)
            }

            fn checked_ceil(self) -> Option<$int> {
                checked_impl!(self.ceil(), $int)
            }

            fn checked_round(self) -> Option<$int> {
                checked_impl!(self.round(), $int)
            }

            fn saturated_floor(self) -> $int {
                saturated_impl!(self.floor(), $int)
            }

            fn saturated_ceil(self) -> $int {
                saturated_impl!(self.ceil(), $int)
            }

            fn saturated_round(self) -> $int {
                saturated_impl!(self.round(), $int)
            }
        })+
    };
}

impl_float_convert!(f32, u8, u16, u32, u64, u128);
impl_float_convert!(f32, i8, i16, i32, i64, i128);
impl_float_convert!(f32, usize, isize);

impl_float_convert!(f64, u8, u16, u32, u64, u128);
impl_float_convert!(f64, i8, i16, i32, i64, i128);
impl_float_convert!(f64, usize, isize);
