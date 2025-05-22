use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use bigdecimal::{
    BigDecimal,
    num_bigint::{BigInt, ToBigInt},
};

use crate::{
    errors::GeneralError,
    prelude::{IntoEndian, U256},
};

/// Represents a general currency - like a token or a coin.
pub trait Currency: Display + FromStr<Err = GeneralError> + Default {
    /// Base unit exponent used for the currency.
    const SCALE: usize;
    /// Name of the currency.
    const NAME: &'static str;
}

/// Represents wxHOPR token [`Currency`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WxHOPR;

impl Display for WxHOPR {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}

impl FromStr for WxHOPR {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case(Self::NAME) {
            Ok(Self)
        } else {
            Err(GeneralError::ParseError("invalid currency name".into()))
        }
    }
}

impl Currency for WxHOPR {
    const NAME: &'static str = "wxHOPR";
    const SCALE: usize = 18;
}

/// Represents xDai coin [`Currency`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct XDai;

impl Display for XDai {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAME)
    }
}

impl FromStr for XDai {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case(Self::NAME) {
            Ok(Self)
        } else {
            Err(GeneralError::ParseError("invalid currency name".into()))
        }
    }
}

impl Currency for XDai {
    const NAME: &'static str = "xDai";
    const SCALE: usize = 18;
}

/// Represents a non-negative balance of some [`Currency`].
///
/// The value is internally always stored in `wei`.
/// When printed as string using the standard ` Display ` implementation, it is formatted in `wei`.
///
/// To print it in human-readable format (in base units instead of `wei`), use [`Balance::to_formatted_string`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Balance<C: Currency>(U256, C);

impl<C: Currency> Display for Balance<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}{}", self.0, Self::WEI_PREFIX, self.1)
    }
}

impl<C: Currency> FromStr for Balance<C> {
    type Err = GeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = regex::Regex::new(&format!("^([\\d\\s.]*\\d)\\s+({}[_\\s]?)?([A-z]+)$", Self::WEI_PREFIX))
            .map_err(|_| GeneralError::ParseError("invalid balance regex".into()))?
            .captures(s)
            .ok_or(GeneralError::ParseError("cannot parse balance".into()))?;

        let mut value = BigDecimal::from_str(&captures[1].replace(' ', ""))
            .map_err(|_| GeneralError::ParseError("invalid balance value".into()))?;

        // If the value is not given in wei, it must be multiplied by the scale
        if captures.get(2).is_none() {
            value *= BigInt::from(10).pow(C::SCALE as u32);
        }

        let value = U256::from_big_endian(
            &value
                .to_bigint()
                .and_then(|b| b.to_biguint())
                .expect("conversion to big unsigned integer never fails")
                .to_bytes_be(),
        );

        Ok(Self(value, C::from_str(&captures[3])?))
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
    const WEI_PREFIX: &'static str = "wei";

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

    /// Returns the amount in base units (human-readable), not in `wei`.
    pub fn to_formatted_string(&self) -> String {
        let dec = BigDecimal::from_biguint(
            bigdecimal::num_bigint::BigUint::from_bytes_be(&self.0.to_be_bytes()),
            C::SCALE as i64,
        );
        let str = dec.to_plain_string();
        if dec.fractional_digit_count() > 0 {
            // Trim excess zeroes if any
            format!("{} {}", str.trim_end_matches('0').trim_end_matches('.'), self.1)
        } else {
            format!("{str} {}", self.1)
        }
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

pub type HoprBalance = Balance<WxHOPR>;

pub type XDaiBalance = Balance<XDai>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::U256;

    #[test]
    fn balance_is_zero_when_zero() {
        assert!(HoprBalance::zero().is_zero());
        assert!(HoprBalance::zero().amount().is_zero());
        assert!(!HoprBalance::from(1).is_zero())
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
    fn balance_should_not_parse_from_different_units() {
        assert!(HoprBalance::from_str(&XDaiBalance::from(10).to_string()).is_err());
    }

    #[test]
    fn balance_should_parse_from_non_wei_string() -> anyhow::Result<()> {
        let balance = HoprBalance::from_str("5 wxHOPR")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE));

        let balance = HoprBalance::from_str("5 wxhopr")?;
        assert_eq!(balance.amount(), U256::from(5) * U256::exp10(WxHOPR::SCALE));

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
    fn balance_test_formatted_string() -> anyhow::Result<()> {
        let base = U256::from(123) * U256::exp10(WxHOPR::SCALE - 2);

        let b1 = format!("{base} wei_wxHOPR");
        let b1: HoprBalance = b1.parse()?;

        let b2 = b1.mul(100);

        let b3 = format!("{} wei_wxHOPR", base / 1000);
        let b3: HoprBalance = b3.parse()?;

        let b4 = format!("{} wei_wxHOPR", base / 10);
        let b4: HoprBalance = b4.parse()?;

        assert_eq!("1.23 wxHOPR", b1.to_formatted_string());
        assert_eq!("123 wxHOPR", b2.to_formatted_string());
        assert_eq!("0.00123 wxHOPR", b3.to_formatted_string());
        assert_eq!("0.123 wxHOPR", b4.to_formatted_string());

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
