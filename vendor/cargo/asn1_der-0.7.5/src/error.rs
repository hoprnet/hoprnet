use crate::rust::fmt::{ self, Display, Formatter };
#[cfg(not(feature = "no_std"))]
	use crate::rust::error::Error;


/// Creates a static error description with file and line information
#[doc(hidden)]
#[macro_export]
macro_rules! e {
	() => (concat!("@", file!(), ":", line!()));
	($str:expr) => (concat!($str, " @", file!(), ":", line!()));
}

/// Creates an `InOutError` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eio {
	($str:expr) => (
		$crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::InOutError(e!($str)))
	)
}
/// Creates an `InvalidData` variant
#[doc(hidden)]
#[macro_export]
macro_rules! einval {
	($str:expr) => (
		$crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::InvalidData(e!($str)))
	);
}
/// Creates an `Unsupported` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eunsupported {
	($str:expr) => (
		$crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::Unsupported(e!($str)))
	);
}
/// Creates an `Other` variant
#[doc(hidden)]
#[macro_export]
macro_rules! eother {
	($str:expr) => (
		$crate::error::Asn1DerError::new($crate::error::Asn1DerErrorVariant::Other(e!($str)))
	);
}


/// A trait to chain errors
pub trait ErrorChain {
	/// Chains another error to `self`
	///
	/// _Info: does nothing if build with `no_std`_
	fn propagate(self, desc: &'static str) -> Self;
}
impl<T> ErrorChain for Result<T, Asn1DerError> {
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn propagate(self, _desc: &'static str) -> Self {
		#[cfg(any(feature = "no_std", feature = "no_panic"))]
			return self;
		#[cfg(not(any(feature = "no_std", feature = "no_panic")))] {
			self.map_err(|e| {
				let new_error = match e.error {
					Asn1DerErrorVariant::InOutError(_) => Asn1DerErrorVariant::InOutError(_desc),
					Asn1DerErrorVariant::InvalidData(_) => Asn1DerErrorVariant::InvalidData(_desc),
					Asn1DerErrorVariant::Unsupported(_) => Asn1DerErrorVariant::Unsupported(_desc),
					Asn1DerErrorVariant::Other(_) => Asn1DerErrorVariant::Other(_desc)
				};
				Asn1DerError{ error: new_error, source: Some(ErrorSource::new(e)) }
			})
		}
	}
}


/// An `Asn1DerError` variant
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Asn1DerErrorVariant {
	/// An in-out error occurred (e.g. failed to read/write some bytes)
	InOutError(&'static str),
	/// The data has an invalid encoding
	InvalidData(&'static str),
	/// The object type or length is not supported by this implementation
	Unsupported(&'static str),
	/// An unspecified error
	Other(&'static str)
}
impl Asn1DerErrorVariant {
	/// Writes the error kind and description to a formatter
	///
	/// _#implicit_validation: we use a `#[inline(never)] extern "C" fn` here to hide this code from
	/// `no_panic` because either way we have to assume that the code in the standard library is
	/// correct – see also
	/// [Is there a way to use potentially panicking code in a #[no_panic] function #16](https://github.com/dtolnay/no-panic/issues/16)_
	#[inline(never)] #[allow(improper_ctypes_definitions)]
	extern "C" fn write(f: &mut Formatter, kind: &str, desc: &str) -> fmt::Result {
		write!(f, "{}: {}", kind, desc)
	}
}
impl Display for Asn1DerErrorVariant {
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Asn1DerErrorVariant::InOutError(desc) => Self::write(f, "I/O error", desc),
			Asn1DerErrorVariant::InvalidData(desc) => Self::write(f, "Invalid encoding", desc),
			Asn1DerErrorVariant::Unsupported(desc) => Self::write(f, "Unsupported", desc),
			Asn1DerErrorVariant::Other(desc) => Self::write(f, "Other", desc)
		}
	}
}


/// An error source
#[doc(hidden)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ErrorSource {
	#[cfg(any(feature = "no_std", feature = "no_panic"))]
		inner: &'static str,
	#[cfg(not(any(feature = "no_std", feature = "no_panic")))]
		inner: Box<Asn1DerError>
}
impl ErrorSource {
	/// Creates a new error source
	#[cfg(not(any(feature = "no_std", feature = "no_panic")))]
	pub fn new(e: Asn1DerError) -> Self {
		Self{ inner: Box::new(e) }
	}
}


/// An `asn1_der` error
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Asn1DerError {
	#[doc(hidden)]
	pub error: Asn1DerErrorVariant,
	#[doc(hidden)]
	pub source: Option<ErrorSource>
}
impl Asn1DerError {
	/// Creates a new error with `variant`
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	pub fn new(variant: Asn1DerErrorVariant) -> Self {
		Self{ error: variant, source: None }
	}
	
	/// Writes the error variant and source (if any) to a formatter
	///
	/// _#implicit_validation: we use a `#[inline(never)] extern "C" fn` here to hide this code from
	/// `no_panic` because either way we have to assume that the code in the standard library is
	/// correct – see also
	/// [Is there a way to use potentially panicking code in a #[no_panic] function #16](https://github.com/dtolnay/no-panic/issues/16)_
	#[inline(never)] #[allow(improper_ctypes_definitions)]
	extern "C" fn write(f: &mut Formatter, variant: &Asn1DerErrorVariant,
		source: &Option<ErrorSource>) -> fmt::Result
	{
		match source.as_ref() {
			Some(source) => write!(f, "{}\n    caused by: {}", variant, source.inner),
			None => write!(f, "{}", variant)
		}
	}
}
impl Display for Asn1DerError {
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Self::write(f, &self.error, &self.source)
	}
}
#[cfg(not(feature = "no_std"))]
impl Error for Asn1DerError {
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		#[cfg(any(feature = "no_std", feature = "no_panic"))]
			return None;
		#[cfg(not(any(feature = "no_std", feature = "no_panic")))]
			return self.source.as_ref().map(|s| s.inner.as_ref() as _);
	}
}