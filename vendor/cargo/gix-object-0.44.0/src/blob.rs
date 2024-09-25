use std::{convert::Infallible, io};

use crate::{Blob, BlobRef, Kind};

impl<'a> crate::WriteTo for BlobRef<'a> {
    /// Write the blobs data to `out` verbatim.
    fn write_to(&self, out: &mut dyn io::Write) -> io::Result<()> {
        out.write_all(self.data)
    }

    fn kind(&self) -> Kind {
        Kind::Blob
    }

    fn size(&self) -> u64 {
        self.data.len() as u64
    }
}

impl crate::WriteTo for Blob {
    /// Write the blobs data to `out` verbatim.
    fn write_to(&self, out: &mut dyn io::Write) -> io::Result<()> {
        self.to_ref().write_to(out)
    }

    fn kind(&self) -> Kind {
        Kind::Blob
    }

    fn size(&self) -> u64 {
        self.to_ref().size()
    }
}

impl Blob {
    /// Provide a `BlobRef` to this owned blob
    pub fn to_ref(&self) -> BlobRef<'_> {
        BlobRef { data: &self.data }
    }
}

impl<'a> BlobRef<'a> {
    /// Instantiate a `Blob` from the given `data`, which is used as-is.
    pub fn from_bytes(data: &[u8]) -> Result<BlobRef<'_>, Infallible> {
        Ok(BlobRef { data })
    }
}
