//! Various helpful macros related to implementing `SerHex`.

/// implement `SerHexSeq` for a specified type.
#[macro_export]
macro_rules! impl_serhex_seq {
    ($type: ty, $bytes: expr) => {
        impl $crate::SerHexSeq<$crate::Strict> for $type {
            fn size() -> usize {
                $bytes
            }
        }

        impl $crate::SerHexSeq<$crate::StrictPfx> for $type {
            fn size() -> usize {
                $bytes
            }
        }

        impl $crate::SerHexSeq<$crate::StrictCap> for $type {
            fn size() -> usize {
                $bytes
            }
        }

        impl $crate::SerHexSeq<$crate::StrictCapPfx> for $type {
            fn size() -> usize {
                $bytes
            }
        }
    };
}

/// helper macro for implementing the `into_hex_raw` function for
/// bytearray-style types.
#[doc(hidden)]
#[macro_export]
macro_rules! into_hex_bytearray {
    ($src: ident, $dst: ident, $len: expr) => {{
        let src: &[u8] = $src.as_ref();
        debug_assert!(src.len() == $len);
        // add prefix if we are doing such things.
        if <C as $crate::HexConf>::withpfx() {
            $dst.write_all("0x".as_bytes())?;
        }
        // if
        if <C as $crate::HexConf>::compact() {
            // find index and location of first non-zero byte.
            if let Some((idx, val)) = src.iter().enumerate().find(|&(_, v)| *v > 0u8) {
                // if first non-zero byte is less than `0x10`, repr w/ one hex char.
                if *val < 0x10 {
                    if <C as $crate::HexConf>::withcap() {
                        $dst.write_all(&[$crate::utils::fromvalcaps(*val)])?;
                        $crate::utils::writehexcaps(&src[(idx + 1)..], $dst)
                    } else {
                        $dst.write_all(&[$crate::utils::fromval(*val)])?;
                        $crate::utils::writehex(&src[(idx + 1)..], $dst)
                    }
                } else {
                    if <C as $crate::HexConf>::withcap() {
                        $crate::utils::writehexcaps(&src[idx..], $dst)
                    } else {
                        $crate::utils::writehex(&src[idx..], $dst)
                    }
                }
            // if no non-zero byte was found, just write in a zero.
            } else {
                $dst.write_all(&[b'0'])?;
                Ok(())
            }
        } else {
            if <C as $crate::HexConf>::withcap() {
                $crate::utils::writehexcaps(src, $dst)
            } else {
                $crate::utils::writehex(src, $dst)
            }
        }
    }};
}

/// helper macro for implementing the `into_hex_raw` function for
/// bytearray-style types.
#[doc(hidden)]
#[macro_export]
macro_rules! from_hex_bytearray {
    ($src: ident, $len: expr) => {{
        let raw: &[u8] = $src.as_ref();
        let hex = if <C as $crate::HexConf>::withpfx() {
            let pfx = "0x".as_bytes();
            if raw.starts_with(pfx) {
                &raw[2..]
            } else {
                raw
            }
        } else {
            raw
        };
        let mut buf = [0u8; $len];
        if <C as $crate::HexConf>::compact() {
            let min = 1;
            let max = $len * 2;
            let got = hex.len();
            if got < min || got > max {
                let inner = $crate::types::ParseHexError::Range { min, max, got };
                let error = $crate::types::Error::from(inner);
                return Err(error.into());
            }
            let body = $len - (got / 2);
            let head = got % 2;
            if head > 0 {
                buf[body - head] = $crate::utils::intobyte(b'0', hex[0])?;
            }
            $crate::utils::fromhex(&mut buf[body..], &hex[head..])?;
        } else {
            $crate::utils::fromhex(&mut buf[..], hex)?;
        }
        Ok(buf)
    }};
}

/// macro for implementing `SerHex` for a type which implements
/// `From<[u8;n]>` and `AsRef<[u8]>`.
#[macro_export]
macro_rules! impl_serhex_bytearray {
    ($type: ty, $len: expr) => {
        impl_serhex_seq!($type, $len);
        impl<C> $crate::SerHex<C> for $type
        where
            C: $crate::HexConf,
        {
            type Error = $crate::types::Error;
            fn into_hex_raw<D>(&self, mut dst: D) -> ::std::result::Result<(), Self::Error>
            where
                D: ::std::io::Write,
            {
                into_hex_bytearray!(self, dst, $len)?;
                Ok(())
            }
            fn from_hex_raw<S>(src: S) -> ::std::result::Result<Self, Self::Error>
            where
                S: AsRef<[u8]>,
            {
                let rslt: ::std::result::Result<[u8; $len], Self::Error> =
                    from_hex_bytearray!(src, $len);
                match rslt {
                    Ok(buf) => Ok(buf.into()),
                    Err(e) => Err(e),
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use {
        Compact, CompactCap, CompactCapPfx, CompactPfx, SerHex, Strict, StrictCap, StrictCapPfx,
        StrictPfx,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct Foo([u8; 4]);
    impl_newtype_bytearray!(Foo, 4);
    impl_serhex_bytearray!(Foo, 4);

    #[test]
    fn hex_strict_ok() {
        let f1 = Foo([0, 1, 2, 3]);
        let hs = <Foo as SerHex<Strict>>::into_hex(&f1).unwrap();
        let f2 = <Foo as SerHex<Strict>>::from_hex(&hs).unwrap();
        assert_eq!(f1, f2);
    }

    #[test]
    #[should_panic]
    fn hex_strict_err() {
        let _ = <Foo as SerHex<Strict>>::from_hex("faaffaa").unwrap();
    }

    #[test]
    fn hex_compact() {
        let f1 = Foo([0, 0, 0x0a, 0xff]);
        let hs = <Foo as SerHex<Compact>>::into_hex(&f1).unwrap();
        assert_eq!(&hs, "aff");
        let f2 = <Foo as SerHex<Compact>>::from_hex(&hs).unwrap();
        assert_eq!(f1, f2);
    }

    #[test]
    fn hex_variants() {
        let f = Foo([0x00, 0x0f, 0xff, 0x11]);
        assert_eq!(
            "0x000fff11",
            <Foo as SerHex<StrictPfx>>::into_hex(&f).unwrap()
        );
        assert_eq!(
            "000FFF11",
            <Foo as SerHex<StrictCap>>::into_hex(&f).unwrap()
        );
        assert_eq!(
            "0x000FFF11",
            <Foo as SerHex<StrictCapPfx>>::into_hex(&f).unwrap()
        );
        assert_eq!(
            "0xfff11",
            <Foo as SerHex<CompactPfx>>::into_hex(&f).unwrap()
        );
        assert_eq!("FFF11", <Foo as SerHex<CompactCap>>::into_hex(&f).unwrap());
        assert_eq!(
            "0xFFF11",
            <Foo as SerHex<CompactCapPfx>>::into_hex(&f).unwrap()
        );
    }

    #[test]
    fn blanket_array() {
        let v: [Foo; 2] = <[Foo; 2] as SerHex<StrictPfx>>::from_hex("0xffaaffaa11221122").unwrap();
        assert_eq!(v[0], Foo([0xff, 0xaa, 0xff, 0xaa]));
        assert_eq!(v[1], Foo([0x11, 0x22, 0x11, 0x22]));
        let hs = <[Foo; 2] as SerHex<StrictPfx>>::into_hex(&v).unwrap();
        assert_eq!(hs, "0xffaaffaa11221122");
    }
}
