//! Useful macros for implementing new chains

#[macro_export]
/// Implement `serde::Serialize` and `serde::Deserialize` by passing through to the hex
macro_rules! impl_hex_serde {
    ($item:ty) => {
        impl serde::Serialize for $item {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let s = $crate::ser::ByteFormat::serialize_hex(self);
                serializer.serialize_str(&s)
            }
        }

        impl<'de> serde::Deserialize<'de> for $item {
            fn deserialize<D>(deserializer: D) -> Result<$item, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s: &str = serde::Deserialize::deserialize(deserializer)?;
                <$item as $crate::ser::ByteFormat>::deserialize_hex(s)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))
            }
        }
    };
}

#[macro_export]
/// Wrap a prefixed vector of bytes (`u8`) in a newtype, and implement convenience functions for
/// it.
macro_rules! wrap_prefixed_byte_vector {
    (
        $(#[$outer:meta])*
        $wrapper_name:ident
    ) => {
        $(#[$outer])*
        #[derive(Clone, Debug, Eq, PartialEq, Default, Hash, PartialOrd, Ord)]
        pub struct $wrapper_name(Vec<u8>);

        impl $crate::ser::ByteFormat for $wrapper_name {
            type Error = $crate::ser::SerError;

            fn serialized_length(&self) -> usize {
                let mut length = self.len();
                length += self.len_prefix() as usize;
                length
            }

            fn read_from<R>(reader: &mut R) -> Result<Self, Self::Error>
            where
                R: std::io::Read
            {
                Ok(coins_core::ser::read_prefix_vec(reader)?.into())
            }

            fn write_to<W>(&self, writer: &mut W) -> Result<usize, Self::Error>
            where
                W: std::io::Write
            {
                coins_core::ser::write_prefix_vec(writer, &self.0)
            }
        }

        impl_hex_serde!($wrapper_name);

        impl std::convert::AsRef<[u8]> for $wrapper_name {
            fn as_ref(&self) -> &[u8] {
                &self.0[..]
            }
        }

        impl $wrapper_name {
            /// Instantate a new wrapped vector
            pub fn new(v: Vec<u8>) -> Self {
                Self(v)
            }

            /// Construct an empty wrapped vector instance.
            pub fn null() -> Self {
                Self(vec![])
            }

            /// Return a reference to the underlying bytes
            pub fn items(&self) -> &[u8] {
                &self.0
            }

            /// Set the underlying items vector.
            pub fn set_items(&mut self, v: Vec<u8>) {
                self.0 = v
            }

            /// Push an item to the item vector.
            pub fn push(&mut self, i: u8) {
                self.0.push(i)
            }

            /// Return the length of the item vector.
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Return true if the length of the item vector is 0.
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }

            /// Determine the byte-length of the vector length prefix
            pub fn len_prefix(&self) -> u8 {
                $crate::ser::prefix_byte_len(self.len() as u64)
            }

            /// Insert an item at the specified index.
            pub fn insert(&mut self, index: usize, i: u8) {
                self.0.insert(index, i)
            }
        }

        impl From<&[u8]> for $wrapper_name {
            fn from(v: &[u8]) -> Self {
                Self(v.to_vec())
            }
        }

        impl From<Vec<u8>> for $wrapper_name {
            fn from(v: Vec<u8>) -> Self {
                Self(v)
            }
        }

        impl std::ops::Index<usize> for $wrapper_name {
            type Output = u8;

            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl std::ops::Index<std::ops::Range<usize>> for $wrapper_name {
            type Output = [u8];

            fn index(&self, range: std::ops::Range<usize>) -> &[u8] {
                &self.0[range]
            }
        }

        impl std::ops::IndexMut<usize> for $wrapper_name {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.0[index]
            }
        }

        impl std::iter::Extend<u8> for $wrapper_name {
            fn extend<I: std::iter::IntoIterator<Item=u8>>(&mut self, iter: I) {
                self.0.extend(iter)
            }
        }

        impl std::iter::IntoIterator for $wrapper_name {
            type Item = u8;
            type IntoIter = std::vec::IntoIter<u8>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }
    }
}

#[macro_export]
/// Implement conversion between script types by passing via `as_ref().into()`
macro_rules! impl_script_conversion {
    ($t1:ty, $t2:ty) => {
        impl From<&$t2> for $t1 {
            fn from(t: &$t2) -> $t1 {
                t.as_ref().into()
            }
        }
        impl From<&$t1> for $t2 {
            fn from(t: &$t1) -> $t2 {
                t.as_ref().into()
            }
        }
    };
}

#[macro_export]
/// Instantiate a new marked digest. Wraps the output of some type that implemented `digest::Digest`
macro_rules! marked_digest {
    (
        $(#[$outer:meta])*
        $marked_name:ident, $digest:ty
    ) => {
        $(#[$outer])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash, PartialOrd, Ord)]
        pub struct $marked_name($crate::hashes::DigestOutput<$digest>);

        impl $marked_name {
            /// Unwrap the marked digest, returning the underlying `GenericArray`
            pub fn to_internal(self) -> $crate::hashes::DigestOutput<$digest> {
                self.0
            }
        }

        impl $crate::hashes::MarkedDigestOutput for $marked_name {
            fn size(&self) -> usize {
                self.0.len()
            }
        }

        impl<T> From<T> for $marked_name
        where
            T: Into<$crate::hashes::DigestOutput<$digest>>
        {
            fn from(t: T) -> Self {
                $marked_name(t.into())
            }
        }

        impl $crate::hashes::MarkedDigest<$marked_name> for $digest {
            fn finalize_marked(self) -> $marked_name {
                $marked_name($crate::hashes::Digest::finalize(self))
            }

            fn digest_marked(data: &[u8]) -> $marked_name {
                $marked_name(<$digest as $crate::hashes::Digest>::digest(data))
            }
        }

        impl AsRef<$crate::hashes::DigestOutput<$digest>> for $marked_name {
            fn as_ref(&self) -> &$crate::hashes::DigestOutput<$digest> {
                &self.0
            }
        }

        impl AsMut<$crate::hashes::DigestOutput<$digest>> for $marked_name {
            fn as_mut(&mut self) -> &mut $crate::hashes::DigestOutput<$digest> {
                &mut self.0
            }
        }

        impl AsRef<[u8]> for $marked_name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl AsMut<[u8]> for $marked_name {
            fn as_mut(&mut self) -> &mut [u8] {
                self.0.as_mut()
            }
        }

        impl $crate::ser::ByteFormat for $marked_name {
            type Error = $crate::ser::SerError;

            fn serialized_length(&self) -> usize {
                $crate::hashes::MarkedDigestOutput::size(self)
            }

            fn read_from<R>(reader: &mut R) -> $crate::ser::SerResult<Self>
            where
                R: std::io::Read,
                Self: std::marker::Sized,
            {
                let mut buf = Self::default();
                reader.read_exact(buf.as_mut())?;
                Ok(buf)
            }

            fn write_to<W>(&self, writer: &mut W) -> $crate::ser::SerResult<usize>
            where
                W: std::io::Write,
            {
                Ok(writer.write(self.as_ref())?)
            }
        }
    };
}
