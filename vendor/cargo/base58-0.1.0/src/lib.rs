//! Base58-to-text encoding
//!
//! Based on https://github.com/trezor/trezor-crypto/blob/master/base58.c
//! commit hash: c6e7d37
//! works only up to 128 bytes

const ALPHABET: &'static [u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

const B58_DIGITS_MAP: &'static [i8] = &[
	-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
	-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
	-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
	-1, 0, 1, 2, 3, 4, 5, 6, 7, 8,-1,-1,-1,-1,-1,-1,
	-1, 9,10,11,12,13,14,15,16,-1,17,18,19,20,21,-1,
	22,23,24,25,26,27,28,29,30,31,32,-1,-1,-1,-1,-1,
	-1,33,34,35,36,37,38,39,40,41,42,43,-1,44,45,46,
	47,48,49,50,51,52,53,54,55,56,57,-1,-1,-1,-1,-1,
];

/// Errors that can occur when decoding base58 encoded string.
#[derive(Debug, PartialEq)]
pub enum FromBase58Error {
	/// The input contained a character which is not a part of the base58 format.
	InvalidBase58Character(char, usize),
	/// The input had invalid length.
	InvalidBase58Length,
}

/// A trait for converting a value to base58 encoded string.
pub trait ToBase58 {
	/// Converts a value of `self` to a base58 value, returning the owned string.
	fn to_base58(&self) -> String;
}

/// A trait for converting base58 encoded values.
pub trait FromBase58 {
	/// Convert a value of `self`, interpreted as base58 encoded data, into an owned vector of bytes, returning a vector.
	fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error>;
}

impl ToBase58 for [u8] {
	fn to_base58(&self) -> String {
		let zcount = self.iter().take_while(|x| **x == 0).count();
		let size = (self.len() - zcount) * 138 / 100 + 1;
		let mut buffer = vec![0u8; size];

		let mut i = zcount;
		let mut high = size - 1;

		while i < self.len() {
			let mut carry = self[i] as u32;
			let mut j = size - 1;

			while j > high || carry != 0 {
				carry += 256 * buffer[j] as u32;
				buffer[j] = (carry % 58) as u8;
				carry /= 58;

				// in original trezor implementation it was underflowing
				if j  > 0 {
					j -= 1;
				}
			}

			i += 1;
			high = j;
		}

		let mut j = buffer.iter().take_while(|x| **x == 0).count();

		let mut result = String::new();
		for _ in 0..zcount {
			result.push('1');
		}

		while j < size {
			result.push(ALPHABET[buffer[j] as usize] as char);
			j += 1;
		}

		result
	}
}



impl FromBase58 for str {
	fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error> {
		let mut bin = [0u8; 132];
		let mut out = [0u32; (132 + 3) / 4];
		let bytesleft = (bin.len() % 4) as u8;
		let zeromask = match bytesleft {
			0 => 0u32,
			_ => 0xffffffff << (bytesleft * 8),
		};

		let zcount = self.chars().take_while(|x| *x == '1').count();
		let mut i = zcount;
		let b58: Vec<u8> = self.bytes().collect();

		while i < self.len() {
			if (b58[i] & 0x80) != 0 {
				// High-bit set on invalid digit
				return Err(FromBase58Error::InvalidBase58Character(b58[i] as char, i));
			}

			if B58_DIGITS_MAP[b58[i] as usize] == -1 {
				// // Invalid base58 digit
				return Err(FromBase58Error::InvalidBase58Character(b58[i] as char, i));
			}

			let mut c = B58_DIGITS_MAP[b58[i] as usize] as u64;
			let mut j = out.len();
			while j != 0 {
				j -= 1;
				let t = out[j] as u64 * 58 + c;
				c = (t & 0x3f00000000) >> 32;
				out[j] = (t & 0xffffffff) as u32;
			}

			if c != 0 {
				// Output number too big (carry to the next int32)
				return Err(FromBase58Error::InvalidBase58Length);
			}

			if (out[0] & zeromask) != 0 {
				// Output number too big (last int32 filled too far)
				return Err(FromBase58Error::InvalidBase58Length);
			}

			i += 1;
		}

		let mut i = 1;
		let mut j = 0;

		bin[0] = match bytesleft {
			3 => ((out[0] & 0xff0000) >> 16) as u8,
			2 => ((out[0] & 0xff00) >> 8) as u8,
			1 => {
				j = 1;
				(out[0] & 0xff) as u8
			},
			_ => {
				i = 0;
				bin[0]
			}
		};

		while j < out.len() {
			bin[i] = ((out[j] >> 0x18) & 0xff) as u8;
			bin[i + 1] = ((out[j] >> 0x10) & 0xff) as u8;
			bin[i + 2] = ((out[j] >> 8) & 0xff) as u8;
			bin[i + 3] = ((out[j] >> 0) & 0xff) as u8;
			i += 4;
			j += 1;
		}

		let leading_zeros = bin.iter().take_while(|x| **x == 0).count();
		Ok(bin[leading_zeros - zcount..].to_vec())
	}
}

#[cfg(test)]
mod tests {
	use super::{ToBase58, FromBase58};

	#[test]
	fn test_from_base58_basic() {
		assert_eq!("".from_base58().unwrap(), b"");
		assert_eq!("Z".from_base58().unwrap(), &[32]);
		assert_eq!("n".from_base58().unwrap(), &[45]);
		assert_eq!("q".from_base58().unwrap(), &[48]);
		assert_eq!("r".from_base58().unwrap(), &[49]);
		assert_eq!("z".from_base58().unwrap(), &[57]);
		assert_eq!("4SU".from_base58().unwrap(), &[45, 49]);
		assert_eq!("4k8".from_base58().unwrap(), &[49, 49]);
		assert_eq!("ZiCa".from_base58().unwrap(), &[97, 98, 99]);
		assert_eq!("3mJr7AoUXx2Wqd".from_base58().unwrap(), b"1234598760");
		assert_eq!("3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f".from_base58().unwrap(), b"abcdefghijklmnopqrstuvwxyz");
	}

	#[test]
	fn test_from_base58_invalid_char() {
		assert!("0".from_base58().is_err());
		assert!("O".from_base58().is_err());
		assert!("I".from_base58().is_err());
		assert!("l".from_base58().is_err());
		assert!("3mJr0".from_base58().is_err());
		assert!("O3yxU".from_base58().is_err());
		assert!("3sNI".from_base58().is_err());
		assert!("4kl8".from_base58().is_err());
		assert!("s!5<".from_base58().is_err());
		assert!("t$@mX<*".from_base58().is_err());
	}

	#[test]
	fn test_from_base58_initial_zeros() {
		assert_eq!("1ZiCa".from_base58().unwrap(), b"\0abc");
		assert_eq!("11ZiCa".from_base58().unwrap(), b"\0\0abc");
		assert_eq!("111ZiCa".from_base58().unwrap(), b"\0\0\0abc");
		assert_eq!("1111ZiCa".from_base58().unwrap(), b"\0\0\0\0abc");
	}

	#[test]
	fn test_to_base58_basic() {
		assert_eq!(b"".to_base58(), "");
		assert_eq!(&[32].to_base58(), "Z");
		assert_eq!(&[45].to_base58(), "n");
		assert_eq!(&[48].to_base58(), "q");
		assert_eq!(&[49].to_base58(), "r");
		assert_eq!(&[57].to_base58(), "z");
		assert_eq!(&[45, 49].to_base58(), "4SU");
		assert_eq!(&[49, 49].to_base58(), "4k8");
		assert_eq!(b"abc".to_base58(), "ZiCa");
		assert_eq!(b"1234598760".to_base58(), "3mJr7AoUXx2Wqd");
		assert_eq!(b"abcdefghijklmnopqrstuvwxyz".to_base58(), "3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f");
	}

	#[test]
	fn test_to_base58_initial_zeros() {
		assert_eq!(b"\0abc".to_base58(), "1ZiCa");
		assert_eq!(b"\0\0abc".to_base58(), "11ZiCa");
		assert_eq!(b"\0\0\0abc".to_base58(), "111ZiCa");
		assert_eq!(b"\0\0\0\0abc".to_base58(), "1111ZiCa");
	}
}
