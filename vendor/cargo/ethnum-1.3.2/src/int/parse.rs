//! Module implementing parsing for `I256` type.

use crate::int::I256;

impl_from_str! {
    impl FromStr for I256;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::from_str_radix;
    use core::num::IntErrorKind;

    #[test]
    fn from_str() {
        assert_eq!("42".parse::<I256>().unwrap(), 42);
    }

    #[test]
    fn from_str_prefixed() {
        assert_eq!(from_str_radix::<I256>("0b101", 2, Some("0b")).unwrap(), 5);
        assert_eq!(from_str_radix::<I256>("-0xf", 16, Some("0x")).unwrap(), -15);
    }

    #[test]
    fn from_str_errors() {
        assert_eq!(
            from_str_radix::<I256>("", 2, None).unwrap_err().kind(),
            &IntErrorKind::Empty,
        );
        assert_eq!(
            from_str_radix::<I256>("?", 2, None).unwrap_err().kind(),
            &IntErrorKind::InvalidDigit,
        );
        assert_eq!(
            from_str_radix::<I256>("1", 16, Some("0x"))
                .unwrap_err()
                .kind(),
            &IntErrorKind::InvalidDigit,
        );
        assert_eq!(
            from_str_radix::<I256>(
                "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                36,
                None
            )
            .unwrap_err()
            .kind(),
            &IntErrorKind::PosOverflow,
        );
        assert_eq!(
            from_str_radix::<I256>(
                "-zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                36,
                None
            )
            .unwrap_err()
            .kind(),
            &IntErrorKind::NegOverflow,
        );
    }
}
