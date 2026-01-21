use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use bigdecimal::{
    BigDecimal,
    num_bigint::{BigInt, BigUint, ToBigInt},
};

use crate::{
    errors::GeneralError,
    prelude::{IntoEndian, U256},
    traits::UnitaryFloatOps,
};

/// Represents a general currency - like a token or a coin.
pub trait Currency: Display + FromStr<Err = GeneralError> + Default + PartialEq + Eq + PartialOrd + Ord {
    /// Name of the currency.
    const NAME: &'static str;

    /// Base unit exponent used for the currency.
    const SCALE: usize;

    /// Checks if this currency is the same as the one given in the template argument.
    fn is<C: Currency>() -> bool {
        Self::NAME == C::NAME
    }

    /// Returns `Ok(())` if the given string is equal to the currency name.
    fn name_matches(s: &str) -> Result<(), GeneralError> {
        if s.eq_ignore_ascii_case(Self::NAME) {
            Ok(())
        } else {
            Err(GeneralError::ParseError("invalid currency name".into()))
        }
    }
}

/// Represents wxHOPR token [`Currency`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WxHOPR;

impl Display for WxHOPR {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}

impl FromStr for WxHOPR {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::name_matches(s).map(|_| Self)
    }
}

impl Currency for WxHOPR {
    const NAME: &'static str = "wxHOPR";
    const SCALE: usize = 18;
}

/// Represents xDai coin [`Currency`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XDai;

impl Display for XDai {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}

impl FromStr for XDai {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::name_matches(s).map(|_| Self)
    }
}

impl Currency for XDai {
    const NAME: &'static str = "xDai";
    const SCALE: usize = 18;
}

/// Represents a non-negative balance of some [`Currency`].
///
/// The value is internally always stored in `wei` but always printed in human-readable format.
///
/// All arithmetic on this type is implicitly saturating at bounds given by [`U256`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Balance<C: Currency>(U256, C);

const WEI_PREFIX: &str = "wei";

lazy_static::lazy_static! {
    static ref BALANCE_REGEX: regex::Regex = regex::Regex::new(&format!("^([\\d\\s.]*\\d)\\s+({WEI_PREFIX}[_\\s]?)?([A-Za-z]+)$")).unwrap();
}

impl<C: Currency> Display for Balance<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.amount_in_base_units(), self.1)
    }
}

impl<C: Currency> std::fmt::Debug for Balance<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as Display
        write!(f, "{} {}", self.amount_in_base_units(), self.1)
    }
}

impl<C: Currency> FromStr for Balance<C> {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = BALANCE_REGEX
            .captures(s)
            .ok_or(GeneralError::ParseError("cannot parse balance".into()))?;

        // Fail-fast if the currency name is not valid
        let currency = C::from_str(&captures[3])?;

        let mut value = BigDecimal::from_str(&captures[1].replace(' ', ""))
            .map_err(|_| GeneralError::ParseError("invalid balance value".into()))?;

        // If the value is not given in wei, it must be multiplied by the scale
        if captures.get(2).is_none() {
            value *= BigInt::from(10).pow(C::SCALE as u32);
        }

        // This discards any excess fractional digits after 10e-SCALE
        let biguint_val = value
            .to_bigint()
            .and_then(|b| b.to_biguint())
            .expect("conversion to big unsigned integer never fails");

        if biguint_val > BigUint::from_bytes_be(&U256::max_value().to_be_bytes()) {
            return Err(GeneralError::ParseError("balance value out of bounds".into()));
        }

        Ok(Self(U256::from_be_bytes(biguint_val.to_bytes_be()), currency))
    }
}

impl<C: Currency, T: Into<U256>> From<T> for Balance<C> {
    fn from(value: T) -> Self {
        Self(value.into(), C::default())
    }
}

impl<C: Currency> AsRef<U256> for Balance<C> {
    fn as_ref(&self) -> &U256 {
        &self.0
    }
}

impl<C: Currency> Balance<C> {
    /// Creates new balance in base units, instead of `wei`.
    pub fn new_base<T: Into<U256>>(value: T) -> Self {
        Self(value.into() * U256::exp10(C::SCALE), C::default())
    }

    /// Zero balance.
    pub fn zero() -> Self {
        Self(U256::zero(), C::default())
    }

    /// Checks if the balance is zero.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Gets the amount in `wei`.
    pub fn amount(&self) -> U256 {
        self.0
    }

    fn base_amount(&self) -> BigDecimal {
        BigDecimal::from_biguint(
            bigdecimal::num_bigint::BigUint::from_bytes_be(&self.0.to_be_bytes()),
            C::SCALE as i64,
        )
    }

    /// Returns the amount in base units (human-readable), not in `wei`.
    pub fn amount_in_base_units(&self) -> String {
        let dec = self.base_amount();
        let str = dec.to_plain_string();

        // Trim excess zeroes if any
        if dec.fractional_digit_count() > 0 {
            str.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            str
        }
    }

    /// Prints the balance formated in `wei` units.
    pub fn format_in_wei(&self) -> String {
        format!("{} {} {}", self.0, WEI_PREFIX, self.1)
    }
}

impl<C: Currency> IntoEndian<32> for Balance<C> {
    fn from_be_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        Self(U256::from_be_bytes(bytes.as_ref()), C::default())
    }

    fn from_le_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        Self(U256::from_le_bytes(bytes.as_ref()), C::default())
    }

    fn to_le_bytes(self) -> [u8; 32] {
        self.0.to_le_bytes()
    }

    fn to_be_bytes(self) -> [u8; 32] {
        self.0.to_be_bytes()
    }
}

impl<C: Currency> Add for Balance<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0), C::default())
    }
}

impl<C: Currency, T: Into<U256>> Add<T> for Balance<C> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_add(rhs.into()), C::default())
    }
}

impl<C: Currency> AddAssign for Balance<C> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl<C: Currency, T: Into<U256>> AddAssign<T> for Balance<C> {
    fn add_assign(&mut self, rhs: T) {
        self.0 = self.0.saturating_add(rhs.into());
    }
}

impl<C: Currency> Sub for Balance<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0), C::default())
    }
}

impl<C: Currency, T: Into<U256>> Sub<T> for Balance<C> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_sub(rhs.into()), C::default())
    }
}

impl<C: Currency> SubAssign for Balance<C> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl<C: Currency, T: Into<U256>> SubAssign<T> for Balance<C> {
    fn sub_assign(&mut self, rhs: T) {
        self.0 = self.0.saturating_sub(rhs.into());
    }
}

impl<C: Currency> Mul for Balance<C> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_mul(rhs.0), C::default())
    }
}

impl<C: Currency, T: Into<U256>> Mul<T> for Balance<C> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0.saturating_mul(rhs.into()), C::default())
    }
}

impl<C: Currency> MulAssign for Balance<C> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_mul(rhs.0);
    }
}

impl<C: Currency, T: Into<U256>> MulAssign<T> for Balance<C> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 = self.0.saturating_mul(rhs.into());
    }
}

impl<C: Currency> std::iter::Sum for Balance<C> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, x| acc + x)
    }
}

impl<C: Currency> UnitaryFloatOps for Balance<C> {
    fn mul_f64(&self, rhs: f64) -> crate::errors::Result<Self> {
        self.0.mul_f64(rhs).map(|x| Self(x, C::default()))
    }

    fn div_f64(&self, rhs: f64) -> crate::errors::Result<Self> {
        self.0.div_f64(rhs).map(|x| Self(x, C::default()))
    }
}

pub type HoprBalance = Balance<WxHOPR>;

pub type XDaiBalance = Balance<XDai>;

#[cfg(test)]
mod tests {
    use std::ops::Div;

    use super::*;
    use crate::primitives::U256;

    #[test]
    fn balance_is_not_zero_when_not_zero() {
        assert!(!HoprBalance::from(1).is_zero())
    }

    #[test]
    fn balance_zero_is_zero() {
        assert_eq!(HoprBalance::zero(), HoprBalance::from(0));
        assert!(HoprBalance::zero().is_zero());
        assert!(HoprBalance::zero().amount().is_zero());
    }

    #[test]
    fn balance_should_have_zero_default() {
        assert_eq!(HoprBalance::default(), HoprBalance::zero());
        assert!(HoprBalance::default().is_zero());
    }

    #[test]
    fn balance_should_saturate_on_bounds() {
        let b1 = HoprBalance::from(U256::max_value());
        let b2 = b1 + HoprBalance::from(1);
        assert_eq!(b1.amount(), U256::max_value());
        assert!(!b2.is_zero());
        assert_eq!(b2.amount(), U256::max_value());

        let b1 = HoprBalance::zero();
        let b2 = b1 - HoprBalance::from(1);
        assert_eq!(b1.amount(), U256::zero());
        assert!(b2.is_zero());
        assert_eq!(b2.amount(), U256::zero());

        let b1 = HoprBalance::from(U256::max_value());
        let b2 = b1 * HoprBalance::from(2);
        assert_eq!(b1.amount(), U256::max_value());
        assert!(!b2.is_zero());
        assert_eq!(b2.amount(), U256::max_value());
    }

    #[test]
    fn balance_should_print_different_units() {
        let b1: HoprBalance = 10.into();
        let b2: XDaiBalance = 10.into();

        assert_ne!(b1.to_string(), b2.to_string());
    }

    #[test]
    fn balance_parsing_should_fail_with_invalid_units() {
        assert!(HoprBalance::from_str("10").is_err());
        assert!(HoprBalance::from_str("10 wei").is_err());
        assert!(HoprBalance::from_str("10 wai wxHOPR").is_err());
        assert!(HoprBalance::from_str("10 wxxHOPR").is_err());
    }

    #[test]
    fn balance_parsing_should_fail_when_out_of_bounds() {
        let too_big = primitive_types::U512::from(U256::max_value()) + 1;
        assert!(HoprBalance::from_str(&format!("{too_big} wei wxHOPR")).is_err());

        let too_big =
            (primitive_types::U512::from(U256::max_value()) + 1).div(primitive_types::U512::exp10(WxHOPR::SCALE)) + 1;
        assert!(HoprBalance::from_str(&format!("{too_big} wxHOPR")).is_err());
    }

    #[test]
    fn balance_should_discard_excess_fractional_digits() -> anyhow::Result<()> {
        let balance: HoprBalance = "1.12345678901234567891 wxHOPR".parse()?;
        assert_eq!("1.123456789012345678 wxHOPR", balance.to_string());

        let balance: HoprBalance = "123.12345678901234567891 wxHOPR".parse()?;
        assert_eq!("123.123456789012345678 wxHOPR", balance.to_string());

        let balance: HoprBalance = "1.12345678901234567891 wei wxHOPR".parse()?;
        assert_eq!("0.000000000000000001 wxHOPR", balance.to_string());

        Ok(())
    }

    #[test]
    fn balance_should_not_parse_from_different_units() {
        assert!(HoprBalance::from_str(&XDaiBalance::from(10).to_string()).is_err());
    }

    #[test]
    fn balance_should_translate_from_non_wei_units() {
        let balance = HoprBalance::new_base(10);
        assert_eq!(balance.amount(), U256::from(10) * U256::exp10(WxHOPR::SCALE));
        assert_eq!(balance.amount_in_base_units(), "10");
    }

    #[test]
    fn balance_should_parse_from_non_wei_string() -> anyhow::Result<()> {
        let balance = HoprBalance::from_str("5 wxHOPR")?;
        assert_eq!(balance, Balance::new_base(5));

        let balance = HoprBalance::from_str("5 wxhopr")?;
        assert_eq!(balance, Balance::new_base(5));

        let balance = HoprBalance::from_str(".5 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        let balance = HoprBalance::from_str(" .5 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        let balance = HoprBalance::from_str("0.5 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        let balance = HoprBalance::from_str("0. 5 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        let balance = HoprBalance::from_str("0. 50 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        let balance = HoprBalance::from_str("0. 5 0 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE - 1));

        Ok(())
    }

    #[test]
    fn balance_should_parse_from_wei_string() -> anyhow::Result<()> {
        let balance = HoprBalance::from_str("5 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str(" 5 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str("5 000 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5000.into());

        let balance = HoprBalance::from_str("5 0 0 0 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5000.into());

        let balance = HoprBalance::from_str("5 wei_wxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str("5 wei wxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str("5 wei wxhopr")?;
        assert_eq!(balance.amount(), 5.into());

        Ok(())
    }

    #[test]
    fn balance_should_parse_from_formatted_string() -> anyhow::Result<()> {
        let balance = HoprBalance::from_str("5.0123 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(50123) * U256::exp10(WxHOPR::SCALE - 4));

        let balance = HoprBalance::from_str("5.001 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str("5.00 weiwxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        let balance = HoprBalance::from_str("5.00 wei_wxHOPR")?;
        assert_eq!(balance.amount(), 5.into());

        Ok(())
    }

    #[test]
    fn balance_should_have_consistent_display_from_str() -> anyhow::Result<()> {
        let balance_1 = HoprBalance::from(10);
        let balance_2 = HoprBalance::from_str(&balance_1.to_string())?;

        assert_eq!(balance_1, balance_2);

        Ok(())
    }

    #[test]
    fn balance_should_have_consistent_formatted_string_from_str() -> anyhow::Result<()> {
        let balance_1 = HoprBalance::from(10);
        let balance_2 = HoprBalance::from_str(&balance_1.format_in_wei())?;

        assert_eq!(balance_1, balance_2);

        Ok(())
    }

    #[test]
    fn balance_test_formatted_string() -> anyhow::Result<()> {
        let base = U256::from(123) * U256::exp10(WxHOPR::SCALE - 2);

        let b1 = format!("{base} wei_wxHOPR");
        let b1: HoprBalance = b1.parse()?;

        let b2 = b1.mul(100);

        let b3 = format!("{} wei_wxHOPR", base / 1000);
        let b3: HoprBalance = b3.parse()?;

        let b4 = format!("{} wei_wxHOPR", base / 10);
        let b4: HoprBalance = b4.parse()?;

        assert_eq!("1.23 wxHOPR", b1.to_string());
        assert_eq!("123 wxHOPR", b2.to_string());
        assert_eq!("0.00123 wxHOPR", b3.to_string());
        assert_eq!("0.123 wxHOPR", b4.to_string());

        Ok(())
    }

    #[test]
    fn balance_should_sum_in_interator_correctly() {
        let sum = vec![HoprBalance::from(1), HoprBalance::from(2), HoprBalance::from(3)]
            .into_iter()
            .sum::<HoprBalance>();

        assert_eq!(sum, HoprBalance::from(6));
    }
}
