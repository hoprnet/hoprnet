//! Types for reading ZIP archives

#[cfg(feature = "aes-crypto")]
use crate::aes::{AesReader, AesReaderValid};
use crate::compression::{CompressionMethod, Decompressor};
use crate::cp437::FromCp437;
use crate::crc32::Crc32Reader;
use crate::extra_fields::{ExtendedTimestamp, ExtraField};
use crate::read::zip_archive::{Shared, SharedBuilder};
use crate::result::{ZipError, ZipResult};
use crate::spec::{self, CentralDirectoryEndInfo, DataAndPosition, FixedSizeBlock, Pod};
use crate::types::{
    AesMode, AesVendorVersion, DateTime, System, ZipCentralEntryBlock, ZipFileData,
    ZipLocalEntryBlock,
};
use crate::zipcrypto::{ZipCryptoReader, ZipCryptoReaderValid, ZipCryptoValidator};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::ffi::OsString;
use std::fs::create_dir_all;
use std::io::{self, copy, prelude::*, sink, SeekFrom};
use std::mem;
use std::mem::size_of;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

mod config;

pub use config::*;

/// Provides high level API for reading from a stream.
pub(crate) mod stream;

#[cfg(feature = "lzma")]
pub(crate) mod lzma;

#[cfg(feature = "xz")]
pub(crate) mod xz;

pub(crate) mod magic_finder;

// Put the struct declaration in a private module to convince rustdoc to display ZipArchive nicely
pub(crate) mod zip_archive {
    use indexmap::IndexMap;
    use std::sync::Arc;

    /// Extract immutable data from `ZipArchive` to make it cheap to clone
    #[derive(Debug)]
    pub(crate) struct Shared {
        pub(crate) files: IndexMap<Box<str>, super::ZipFileData>,
        pub(super) offset: u64,
        pub(super) dir_start: u64,
        // This isn't yet used anywhere, but it is here for use cases in the future.
        #[allow(dead_code)]
        pub(super) config: super::Config,
        pub(crate) comment: Box<[u8]>,
        pub(crate) zip64_comment: Option<Box<[u8]>>,
    }

    #[derive(Debug)]
    pub(crate) struct SharedBuilder {
        pub(crate) files: Vec<super::ZipFileData>,
        pub(super) offset: u64,
        pub(super) dir_start: u64,
        // This isn't yet used anywhere, but it is here for use cases in the future.
        #[allow(dead_code)]
        pub(super) config: super::Config,
    }

    impl SharedBuilder {
        pub fn build(self, comment: Box<[u8]>, zip64_comment: Option<Box<[u8]>>) -> Shared {
            let mut index_map = IndexMap::with_capacity(self.files.len());
            self.files.into_iter().for_each(|file| {
                index_map.insert(file.file_name.clone(), file);
            });
            Shared {
                files: index_map,
                offset: self.offset,
                dir_start: self.dir_start,
                config: self.config,
                comment,
                zip64_comment,
            }
        }
    }

    /// ZIP archive reader
    ///
    /// At the moment, this type is cheap to clone if this is the case for the
    /// reader it uses. However, this is not guaranteed by this crate and it may
    /// change in the future.
    ///
    /// ```no_run
    /// use std::io::prelude::*;
    /// fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    ///     use zip::HasZipMetadata;
    ///     let mut zip = zip::ZipArchive::new(reader)?;
    ///
    ///     for i in 0..zip.len() {
    ///         let mut file = zip.by_index(i)?;
    ///         println!("Filename: {}", file.name());
    ///         std::io::copy(&mut file, &mut std::io::stdout())?;
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[derive(Clone, Debug)]
    pub struct ZipArchive<R> {
        pub(super) reader: R,
        pub(super) shared: Arc<Shared>,
    }
}

#[cfg(feature = "aes-crypto")]
use crate::aes::PWD_VERIFY_LENGTH;
use crate::extra_fields::UnicodeExtraField;
use crate::result::ZipError::{InvalidArchive, InvalidPassword};
use crate::spec::is_dir;
use crate::types::ffi::S_IFLNK;
use crate::unstable::{path_to_string, LittleEndianReadExt};
pub use zip_archive::ZipArchive;

#[allow(clippy::large_enum_variant)]
pub(crate) enum CryptoReader<'a> {
    Plaintext(io::Take<&'a mut dyn Read>),
    ZipCrypto(ZipCryptoReaderValid<io::Take<&'a mut dyn Read>>),
    #[cfg(feature = "aes-crypto")]
    Aes {
        reader: AesReaderValid<io::Take<&'a mut dyn Read>>,
        vendor_version: AesVendorVersion,
    },
}

impl Read for CryptoReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            CryptoReader::Plaintext(r) => r.read(buf),
            CryptoReader::ZipCrypto(r) => r.read(buf),
            #[cfg(feature = "aes-crypto")]
            CryptoReader::Aes { reader: r, .. } => r.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match self {
            CryptoReader::Plaintext(r) => r.read_to_end(buf),
            CryptoReader::ZipCrypto(r) => r.read_to_end(buf),
            #[cfg(feature = "aes-crypto")]
            CryptoReader::Aes { reader: r, .. } => r.read_to_end(buf),
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        match self {
            CryptoReader::Plaintext(r) => r.read_to_string(buf),
            CryptoReader::ZipCrypto(r) => r.read_to_string(buf),
            #[cfg(feature = "aes-crypto")]
            CryptoReader::Aes { reader: r, .. } => r.read_to_string(buf),
        }
    }
}

impl<'a> CryptoReader<'a> {
    /// Consumes this decoder, returning the underlying reader.
    pub fn into_inner(self) -> io::Take<&'a mut dyn Read> {
        match self {
            CryptoReader::Plaintext(r) => r,
            CryptoReader::ZipCrypto(r) => r.into_inner(),
            #[cfg(feature = "aes-crypto")]
            CryptoReader::Aes { reader: r, .. } => r.into_inner(),
        }
    }

    /// Returns `true` if the data is encrypted using AE2.
    pub const fn is_ae2_encrypted(&self) -> bool {
        #[cfg(feature = "aes-crypto")]
        return matches!(
            self,
            CryptoReader::Aes {
                vendor_version: AesVendorVersion::Ae2,
                ..
            }
        );
        #[cfg(not(feature = "aes-crypto"))]
        false
    }
}

#[cold]
fn invalid_state<T>() -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "ZipFileReader was in an invalid state",
    ))
}

pub(crate) enum ZipFileReader<'a> {
    NoReader,
    Raw(io::Take<&'a mut dyn Read>),
    Compressed(Box<Crc32Reader<Decompressor<io::BufReader<CryptoReader<'a>>>>>),
}

impl Read for ZipFileReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            ZipFileReader::NoReader => invalid_state(),
            ZipFileReader::Raw(r) => r.read(buf),
            ZipFileReader::Compressed(r) => r.read(buf),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        match self {
            ZipFileReader::NoReader => invalid_state(),
            ZipFileReader::Raw(r) => r.read_exact(buf),
            ZipFileReader::Compressed(r) => r.read_exact(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match self {
            ZipFileReader::NoReader => invalid_state(),
            ZipFileReader::Raw(r) => r.read_to_end(buf),
            ZipFileReader::Compressed(r) => r.read_to_end(buf),
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        match self {
            ZipFileReader::NoReader => invalid_state(),
            ZipFileReader::Raw(r) => r.read_to_string(buf),
            ZipFileReader::Compressed(r) => r.read_to_string(buf),
        }
    }
}

impl<'a> ZipFileReader<'a> {
    fn into_inner(self) -> io::Result<io::Take<&'a mut dyn Read>> {
        match self {
            ZipFileReader::NoReader => invalid_state(),
            ZipFileReader::Raw(r) => Ok(r),
            ZipFileReader::Compressed(r) => {
                Ok(r.into_inner().into_inner().into_inner().into_inner())
            }
        }
    }
}

/// A struct for reading a zip file
pub struct ZipFile<'a> {
    pub(crate) data: Cow<'a, ZipFileData>,
    pub(crate) reader: ZipFileReader<'a>,
}

/// A struct for reading and seeking a zip file
pub struct ZipFileSeek<'a, R> {
    data: Cow<'a, ZipFileData>,
    reader: ZipFileSeekReader<'a, R>,
}

enum ZipFileSeekReader<'a, R> {
    Raw(SeekableTake<'a, R>),
}

struct SeekableTake<'a, R> {
    inner: &'a mut R,
    inner_starting_offset: u64,
    length: u64,
    current_offset: u64,
}

impl<'a, R: Seek> SeekableTake<'a, R> {
    pub fn new(inner: &'a mut R, length: u64) -> io::Result<Self> {
        let inner_starting_offset = inner.stream_position()?;
        Ok(Self {
            inner,
            inner_starting_offset,
            length,
            current_offset: 0,
        })
    }
}

impl<R: Seek> Seek for SeekableTake<'_, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let offset = match pos {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::End(offset) => self.length.checked_add_signed(offset),
            SeekFrom::Current(offset) => self.current_offset.checked_add_signed(offset),
        };
        match offset {
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
            Some(offset) => {
                let clamped_offset = std::cmp::min(self.length, offset);
                let new_inner_offset = self
                    .inner
                    .seek(SeekFrom::Start(self.inner_starting_offset + clamped_offset))?;
                self.current_offset = new_inner_offset - self.inner_starting_offset;
                Ok(new_inner_offset)
            }
        }
    }
}

impl<R: Read> Read for SeekableTake<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let written = self
            .inner
            .take(self.length - self.current_offset)
            .read(buf)?;
        self.current_offset += written as u64;
        Ok(written)
    }
}

pub(crate) fn find_content<'a>(
    data: &ZipFileData,
    reader: &'a mut (impl Read + Seek),
) -> ZipResult<io::Take<&'a mut dyn Read>> {
    // TODO: use .get_or_try_init() once stabilized to provide a closure returning a Result!
    let data_start = match data.data_start.get() {
        Some(data_start) => *data_start,
        None => find_data_start(data, reader)?,
    };

    reader.seek(SeekFrom::Start(data_start))?;
    Ok((reader as &mut dyn Read).take(data.compressed_size))
}

fn find_content_seek<'a, R: Read + Seek>(
    data: &ZipFileData,
    reader: &'a mut R,
) -> ZipResult<SeekableTake<'a, R>> {
    // Parse local header
    let data_start = find_data_start(data, reader)?;
    reader.seek(SeekFrom::Start(data_start))?;

    // Explicit Ok and ? are needed to convert io::Error to ZipError
    Ok(SeekableTake::new(reader, data.compressed_size)?)
}

fn find_data_start(
    data: &ZipFileData,
    reader: &mut (impl Read + Seek + Sized),
) -> Result<u64, ZipError> {
    // Go to start of data.
    reader.seek(SeekFrom::Start(data.header_start))?;

    // Parse static-sized fields and check the magic value.
    let block = ZipLocalEntryBlock::parse(reader)?;

    // Calculate the end of the local header from the fields we just parsed.
    let variable_fields_len =
        // Each of these fields must be converted to u64 before adding, as the result may
        // easily overflow a u16.
        block.file_name_length as u64 + block.extra_field_length as u64;
    let data_start =
        data.header_start + size_of::<ZipLocalEntryBlock>() as u64 + variable_fields_len;

    // Set the value so we don't have to read it again.
    match data.data_start.set(data_start) {
        Ok(()) => (),
        // If the value was already set in the meantime, ensure it matches (this is probably
        // unnecessary).
        Err(_) => {
            debug_assert_eq!(*data.data_start.get().unwrap(), data_start);
        }
    }

    Ok(data_start)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_crypto_reader<'a>(
    data: &ZipFileData,
    reader: io::Take<&'a mut dyn Read>,
    password: Option<&[u8]>,
    aes_info: Option<(AesMode, AesVendorVersion, CompressionMethod)>,
) -> ZipResult<CryptoReader<'a>> {
    #[allow(deprecated)]
    {
        if let CompressionMethod::Unsupported(_) = data.compression_method {
            return unsupported_zip_error("Compression method not supported");
        }
    }

    let reader = match (password, aes_info) {
        #[cfg(not(feature = "aes-crypto"))]
        (Some(_), Some(_)) => {
            return Err(ZipError::UnsupportedArchive(
                "AES encrypted files cannot be decrypted without the aes-crypto feature.",
            ))
        }
        #[cfg(feature = "aes-crypto")]
        (Some(password), Some((aes_mode, vendor_version, _))) => CryptoReader::Aes {
            reader: AesReader::new(reader, aes_mode, data.compressed_size).validate(password)?,
            vendor_version,
        },
        (Some(password), None) => {
            let mut last_modified_time = data.last_modified_time;
            if !data.using_data_descriptor {
                last_modified_time = None;
            }
            let validator = if let Some(last_modified_time) = last_modified_time {
                ZipCryptoValidator::InfoZipMsdosTime(last_modified_time.timepart())
            } else {
                ZipCryptoValidator::PkzipCrc32(data.crc32)
            };
            CryptoReader::ZipCrypto(ZipCryptoReader::new(reader, password).validate(validator)?)
        }
        (None, Some(_)) => return Err(InvalidPassword),
        (None, None) => CryptoReader::Plaintext(reader),
    };
    Ok(reader)
}

pub(crate) fn make_reader(
    compression_method: CompressionMethod,
    crc32: u32,
    reader: CryptoReader,
) -> ZipResult<ZipFileReader> {
    let ae2_encrypted = reader.is_ae2_encrypted();

    Ok(ZipFileReader::Compressed(Box::new(Crc32Reader::new(
        Decompressor::new(io::BufReader::new(reader), compression_method)?,
        crc32,
        ae2_encrypted,
    ))))
}

#[derive(Debug)]
pub(crate) struct CentralDirectoryInfo {
    pub(crate) archive_offset: u64,
    pub(crate) directory_start: u64,
    pub(crate) number_of_files: usize,
    pub(crate) disk_number: u32,
    pub(crate) disk_with_central_directory: u32,
}

impl<'a> TryFrom<&'a CentralDirectoryEndInfo> for CentralDirectoryInfo {
    type Error = ZipError;

    fn try_from(value: &'a CentralDirectoryEndInfo) -> Result<Self, Self::Error> {
        let (relative_cd_offset, number_of_files, disk_number, disk_with_central_directory) =
            match &value.eocd64 {
                Some(DataAndPosition { data: eocd64, .. }) => {
                    if eocd64.number_of_files_on_this_disk > eocd64.number_of_files {
                        return Err(InvalidArchive(
                        "ZIP64 footer indicates more files on this disk than in the whole archive",
                    ));
                    } else if eocd64.version_needed_to_extract > eocd64.version_made_by {
                        return Err(InvalidArchive(
                        "ZIP64 footer indicates a new version is needed to extract this archive than the \
                                 version that wrote it",
                    ));
                    }
                    (
                        eocd64.central_directory_offset,
                        eocd64.number_of_files as usize,
                        eocd64.disk_number,
                        eocd64.disk_with_central_directory,
                    )
                }
                _ => (
                    value.eocd.data.central_directory_offset as u64,
                    value.eocd.data.number_of_files_on_this_disk as usize,
                    value.eocd.data.disk_number as u32,
                    value.eocd.data.disk_with_central_directory as u32,
                ),
            };

        let directory_start = relative_cd_offset
            .checked_add(value.archive_offset)
            .ok_or(InvalidArchive("Invalid central directory size or offset"))?;

        Ok(Self {
            archive_offset: value.archive_offset,
            directory_start,
            number_of_files,
            disk_number,
            disk_with_central_directory,
        })
    }
}

impl<R> ZipArchive<R> {
    pub(crate) fn from_finalized_writer(
        files: IndexMap<Box<str>, ZipFileData>,
        comment: Box<[u8]>,
        zip64_comment: Option<Box<[u8]>>,
        reader: R,
        central_start: u64,
    ) -> ZipResult<Self> {
        let initial_offset = match files.first() {
            Some((_, file)) => file.header_start,
            None => central_start,
        };
        let shared = Arc::new(Shared {
            files,
            offset: initial_offset,
            dir_start: central_start,
            config: Config {
                archive_offset: ArchiveOffset::Known(initial_offset),
            },
            comment,
            zip64_comment,
        });
        Ok(Self { reader, shared })
    }

    /// Total size of the files in the archive, if it can be known. Doesn't include directories or
    /// metadata.
    pub fn decompressed_size(&self) -> Option<u128> {
        let mut total = 0u128;
        for file in self.shared.files.values() {
            if file.using_data_descriptor {
                return None;
            }
            total = total.checked_add(file.uncompressed_size as u128)?;
        }
        Some(total)
    }
}

impl<R: Read + Seek> ZipArchive<R> {
    pub(crate) fn merge_contents<W: Write + Seek>(
        &mut self,
        mut w: W,
    ) -> ZipResult<IndexMap<Box<str>, ZipFileData>> {
        if self.shared.files.is_empty() {
            return Ok(IndexMap::new());
        }
        let mut new_files = self.shared.files.clone();
        /* The first file header will probably start at the beginning of the file, but zip doesn't
         * enforce that, and executable zips like PEX files will have a shebang line so will
         * definitely be greater than 0.
         *
         * assert_eq!(0, new_files[0].header_start); // Avoid this.
         */

        let first_new_file_header_start = w.stream_position()?;

        /* Push back file header starts for all entries in the covered files. */
        new_files.values_mut().try_for_each(|f| {
            /* This is probably the only really important thing to change. */
            f.header_start = f
                .header_start
                .checked_add(first_new_file_header_start)
                .ok_or(InvalidArchive(
                    "new header start from merge would have been too large",
                ))?;
            /* This is only ever used internally to cache metadata lookups (it's not part of the
             * zip spec), and 0 is the sentinel value. */
            f.central_header_start = 0;
            /* This is an atomic variable so it can be updated from another thread in the
             * implementation (which is good!). */
            if let Some(old_data_start) = f.data_start.take() {
                let new_data_start = old_data_start
                    .checked_add(first_new_file_header_start)
                    .ok_or(InvalidArchive(
                        "new data start from merge would have been too large",
                    ))?;
                f.data_start.get_or_init(|| new_data_start);
            }
            Ok::<_, ZipError>(())
        })?;

        /* Rewind to the beginning of the file.
         *
         * NB: we *could* decide to start copying from new_files[0].header_start instead, which
         * would avoid copying over e.g. any pex shebangs or other file contents that start before
         * the first zip file entry. However, zip files actually shouldn't care about garbage data
         * in *between* real entries, since the central directory header records the correct start
         * location of each, and keeping track of that math is more complicated logic that will only
         * rarely be used, since most zips that get merged together are likely to be produced
         * specifically for that purpose (and therefore are unlikely to have a shebang or other
         * preface). Finally, this preserves any data that might actually be useful.
         */
        self.reader.rewind()?;
        /* Find the end of the file data. */
        let length_to_read = self.shared.dir_start;
        /* Produce a Read that reads bytes up until the start of the central directory header.
         * This "as &mut dyn Read" trick is used elsewhere to avoid having to clone the underlying
         * handle, which it really shouldn't need to anyway. */
        let mut limited_raw = (&mut self.reader as &mut dyn Read).take(length_to_read);
        /* Copy over file data from source archive directly. */
        io::copy(&mut limited_raw, &mut w)?;

        /* Return the files we've just written to the data stream. */
        Ok(new_files)
    }

    /// Get the directory start offset and number of files. This is done in a
    /// separate function to ease the control flow design.
    pub(crate) fn get_metadata(config: Config, reader: &mut R) -> ZipResult<Shared> {
        // End of the probed region, initially set to the end of the file
        let file_len = reader.seek(io::SeekFrom::End(0))?;
        let mut end_exclusive = file_len;

        loop {
            // Find the EOCD and possibly EOCD64 entries and determine the archive offset.
            let cde = spec::find_central_directory(
                reader,
                config.archive_offset,
                end_exclusive,
                file_len,
            )?;

            // Turn EOCD into internal representation.
            let Ok(shared) = CentralDirectoryInfo::try_from(&cde)
                .and_then(|info| Self::read_central_header(info, config, reader))
            else {
                // The next EOCD candidate should start before the current one.
                end_exclusive = cde.eocd.position;
                continue;
            };

            return Ok(shared.build(
                cde.eocd.data.zip_file_comment,
                cde.eocd64.map(|v| v.data.extensible_data_sector),
            ));
        }
    }

    fn read_central_header(
        dir_info: CentralDirectoryInfo,
        config: Config,
        reader: &mut R,
    ) -> Result<SharedBuilder, ZipError> {
        // If the parsed number of files is greater than the offset then
        // something fishy is going on and we shouldn't trust number_of_files.
        let file_capacity = if dir_info.number_of_files > dir_info.directory_start as usize {
            0
        } else {
            dir_info.number_of_files
        };

        if dir_info.disk_number != dir_info.disk_with_central_directory {
            return unsupported_zip_error("Support for multi-disk files is not implemented");
        }

        if file_capacity.saturating_mul(size_of::<ZipFileData>()) > isize::MAX as usize {
            return unsupported_zip_error("Oversized central directory");
        }

        let mut files = Vec::with_capacity(file_capacity);
        reader.seek(SeekFrom::Start(dir_info.directory_start))?;
        for _ in 0..dir_info.number_of_files {
            let file = central_header_to_zip_file(reader, &dir_info)?;
            files.push(file);
        }

        Ok(SharedBuilder {
            files,
            offset: dir_info.archive_offset,
            dir_start: dir_info.directory_start,
            config,
        })
    }

    /// Returns the verification value and salt for the AES encryption of the file
    ///
    /// It fails if the file number is invalid.
    ///
    /// # Returns
    ///
    /// - None if the file is not encrypted with AES
    #[cfg(feature = "aes-crypto")]
    pub fn get_aes_verification_key_and_salt(
        &mut self,
        file_number: usize,
    ) -> ZipResult<Option<AesInfo>> {
        let (_, data) = self
            .shared
            .files
            .get_index(file_number)
            .ok_or(ZipError::FileNotFound)?;

        let limit_reader = find_content(data, &mut self.reader)?;
        match data.aes_mode {
            None => Ok(None),
            Some((aes_mode, _, _)) => {
                let (verification_value, salt) =
                    AesReader::new(limit_reader, aes_mode, data.compressed_size)
                        .get_verification_value_and_salt()?;
                let aes_info = AesInfo {
                    aes_mode,
                    verification_value,
                    salt,
                };
                Ok(Some(aes_info))
            }
        }
    }

    /// Read a ZIP archive, collecting the files it contains.
    ///
    /// This uses the central directory record of the ZIP file, and ignores local file headers.
    ///
    /// A default [`Config`] is used.
    pub fn new(reader: R) -> ZipResult<ZipArchive<R>> {
        Self::with_config(Default::default(), reader)
    }

    /// Read a ZIP archive providing a read configuration, collecting the files it contains.
    ///
    /// This uses the central directory record of the ZIP file, and ignores local file headers.
    pub fn with_config(config: Config, mut reader: R) -> ZipResult<ZipArchive<R>> {
        let shared = Self::get_metadata(config, &mut reader)?;

        Ok(ZipArchive {
            reader,
            shared: shared.into(),
        })
    }

    /// Extract a Zip archive into a directory, overwriting files if they
    /// already exist. Paths are sanitized with [`ZipFile::enclosed_name`].
    ///
    /// Extraction is not atomic. If an error is encountered, some of the files
    /// may be left on disk. However, on Unix targets, no newly-created directories with part but
    /// not all of their contents extracted will be readable, writable or usable as process working
    /// directories by any non-root user except you.
    ///
    /// On Unix and Windows, symbolic links are extracted correctly. On other platforms such as
    /// WebAssembly, symbolic links aren't supported, so they're extracted as normal files
    /// containing the target path in UTF-8.
    pub fn extract<P: AsRef<Path>>(&mut self, directory: P) -> ZipResult<()> {
        use std::fs;
        #[cfg(unix)]
        let mut files_by_unix_mode = Vec::new();
        for i in 0..self.len() {
            let mut file = self.by_index(i)?;
            let filepath = file
                .enclosed_name()
                .ok_or(InvalidArchive("Invalid file path"))?;

            let outpath = directory.as_ref().join(filepath);

            if file.is_dir() {
                Self::make_writable_dir_all(&outpath)?;
                continue;
            }
            let symlink_target = if file.is_symlink() && (cfg!(unix) || cfg!(windows)) {
                let mut target = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut target)?;
                Some(target)
            } else {
                None
            };
            drop(file);
            if let Some(p) = outpath.parent() {
                Self::make_writable_dir_all(p)?;
            }
            if let Some(target) = symlink_target {
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    let target = OsString::from_vec(target);
                    std::os::unix::fs::symlink(&target, outpath.as_path())?;
                }
                #[cfg(windows)]
                {
                    let Ok(target) = String::from_utf8(target) else {
                        return Err(ZipError::InvalidArchive("Invalid UTF-8 as symlink target"));
                    };
                    let target = target.into_boxed_str();
                    let target_is_dir_from_archive =
                        self.shared.files.contains_key(&target) && is_dir(&target);
                    let target_path = directory.as_ref().join(OsString::from(target.to_string()));
                    let target_is_dir = if target_is_dir_from_archive {
                        true
                    } else if let Ok(meta) = std::fs::metadata(&target_path) {
                        meta.is_dir()
                    } else {
                        false
                    };
                    if target_is_dir {
                        std::os::windows::fs::symlink_dir(target_path, outpath.as_path())?;
                    } else {
                        std::os::windows::fs::symlink_file(target_path, outpath.as_path())?;
                    }
                }
                continue;
            }
            let mut file = self.by_index(i)?;
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            #[cfg(unix)]
            {
                // Check for real permissions, which we'll set in a second pass
                if let Some(mode) = file.unix_mode() {
                    files_by_unix_mode.push((outpath.clone(), mode));
                }
            }
        }
        #[cfg(unix)]
        {
            use std::cmp::Reverse;
            use std::os::unix::fs::PermissionsExt;

            if files_by_unix_mode.len() > 1 {
                // Ensure we update children's permissions before making a parent unwritable
                files_by_unix_mode.sort_by_key(|(path, _)| Reverse(path.clone()));
            }
            for (path, mode) in files_by_unix_mode.into_iter() {
                fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
            }
        }
        Ok(())
    }

    fn make_writable_dir_all<T: AsRef<Path>>(outpath: T) -> Result<(), ZipError> {
        create_dir_all(outpath.as_ref())?;
        #[cfg(unix)]
        {
            // Dirs must be writable until all normal files are extracted
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                outpath.as_ref(),
                std::fs::Permissions::from_mode(
                    0o700 | std::fs::metadata(outpath.as_ref())?.permissions().mode(),
                ),
            )?;
        }
        Ok(())
    }

    /// Number of files contained in this zip.
    pub fn len(&self) -> usize {
        self.shared.files.len()
    }

    /// Get the starting offset of the zip central directory.
    pub fn central_directory_start(&self) -> u64 {
        self.shared.dir_start
    }

    /// Whether this zip archive contains no files
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the offset from the beginning of the underlying reader that this zip begins at, in bytes.
    ///
    /// Normally this value is zero, but if the zip has arbitrary data prepended to it, then this value will be the size
    /// of that prepended data.
    pub fn offset(&self) -> u64 {
        self.shared.offset
    }

    /// Get the comment of the zip archive.
    pub fn comment(&self) -> &[u8] {
        &self.shared.comment
    }

    /// Get the ZIP64 comment of the zip archive, if it is ZIP64.
    pub fn zip64_comment(&self) -> Option<&[u8]> {
        self.shared.zip64_comment.as_deref()
    }

    /// Returns an iterator over all the file and directory names in this archive.
    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.shared.files.keys().map(|s| s.as_ref())
    }

    /// Search for a file entry by name, decrypt with given password
    ///
    /// # Warning
    ///
    /// The implementation of the cryptographic algorithms has not
    /// gone through a correctness review, and you should assume it is insecure:
    /// passwords used with this API may be compromised.
    ///
    /// This function sometimes accepts wrong password. This is because the ZIP spec only allows us
    /// to check for a 1/256 chance that the password is correct.
    /// There are many passwords out there that will also pass the validity checks
    /// we are able to perform. This is a weakness of the ZipCrypto algorithm,
    /// due to its fairly primitive approach to cryptography.
    pub fn by_name_decrypt(&mut self, name: &str, password: &[u8]) -> ZipResult<ZipFile> {
        self.by_name_with_optional_password(name, Some(password))
    }

    /// Search for a file entry by name
    pub fn by_name(&mut self, name: &str) -> ZipResult<ZipFile> {
        self.by_name_with_optional_password(name, None)
    }

    /// Get the index of a file entry by name, if it's present.
    #[inline(always)]
    pub fn index_for_name(&self, name: &str) -> Option<usize> {
        self.shared.files.get_index_of(name)
    }

    /// Get the index of a file entry by path, if it's present.
    #[inline(always)]
    pub fn index_for_path<T: AsRef<Path>>(&self, path: T) -> Option<usize> {
        self.index_for_name(&path_to_string(path))
    }

    /// Get the name of a file entry, if it's present.
    #[inline(always)]
    pub fn name_for_index(&self, index: usize) -> Option<&str> {
        self.shared
            .files
            .get_index(index)
            .map(|(name, _)| name.as_ref())
    }

    /// Search for a file entry by name and return a seekable object.
    pub fn by_name_seek(&mut self, name: &str) -> ZipResult<ZipFileSeek<R>> {
        self.by_index_seek(self.index_for_name(name).ok_or(ZipError::FileNotFound)?)
    }

    /// Search for a file entry by index and return a seekable object.
    pub fn by_index_seek(&mut self, index: usize) -> ZipResult<ZipFileSeek<R>> {
        let reader = &mut self.reader;
        self.shared
            .files
            .get_index(index)
            .ok_or(ZipError::FileNotFound)
            .and_then(move |(_, data)| {
                let seek_reader = match data.compression_method {
                    CompressionMethod::Stored => {
                        ZipFileSeekReader::Raw(find_content_seek(data, reader)?)
                    }
                    _ => {
                        return Err(ZipError::UnsupportedArchive(
                            "Seekable compressed files are not yet supported",
                        ))
                    }
                };
                Ok(ZipFileSeek {
                    reader: seek_reader,
                    data: Cow::Borrowed(data),
                })
            })
    }

    fn by_name_with_optional_password<'a>(
        &'a mut self,
        name: &str,
        password: Option<&[u8]>,
    ) -> ZipResult<ZipFile<'a>> {
        let Some(index) = self.shared.files.get_index_of(name) else {
            return Err(ZipError::FileNotFound);
        };
        self.by_index_with_optional_password(index, password)
    }

    /// Get a contained file by index, decrypt with given password
    ///
    /// # Warning
    ///
    /// The implementation of the cryptographic algorithms has not
    /// gone through a correctness review, and you should assume it is insecure:
    /// passwords used with this API may be compromised.
    ///
    /// This function sometimes accepts wrong password. This is because the ZIP spec only allows us
    /// to check for a 1/256 chance that the password is correct.
    /// There are many passwords out there that will also pass the validity checks
    /// we are able to perform. This is a weakness of the ZipCrypto algorithm,
    /// due to its fairly primitive approach to cryptography.
    pub fn by_index_decrypt(
        &mut self,
        file_number: usize,
        password: &[u8],
    ) -> ZipResult<ZipFile<'_>> {
        self.by_index_with_optional_password(file_number, Some(password))
    }

    /// Get a contained file by index
    pub fn by_index(&mut self, file_number: usize) -> ZipResult<ZipFile<'_>> {
        self.by_index_with_optional_password(file_number, None)
    }

    /// Get a contained file by index without decompressing it
    pub fn by_index_raw(&mut self, file_number: usize) -> ZipResult<ZipFile<'_>> {
        let reader = &mut self.reader;
        let (_, data) = self
            .shared
            .files
            .get_index(file_number)
            .ok_or(ZipError::FileNotFound)?;
        Ok(ZipFile {
            reader: ZipFileReader::Raw(find_content(data, reader)?),
            data: Cow::Borrowed(data),
        })
    }

    fn by_index_with_optional_password(
        &mut self,
        file_number: usize,
        mut password: Option<&[u8]>,
    ) -> ZipResult<ZipFile<'_>> {
        let (_, data) = self
            .shared
            .files
            .get_index(file_number)
            .ok_or(ZipError::FileNotFound)?;

        match (password, data.encrypted) {
            (None, true) => return Err(ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED)),
            (Some(_), false) => password = None, //Password supplied, but none needed! Discard.
            _ => {}
        }
        let limit_reader = find_content(data, &mut self.reader)?;

        let crypto_reader = make_crypto_reader(data, limit_reader, password, data.aes_mode)?;

        Ok(ZipFile {
            data: Cow::Borrowed(data),
            reader: make_reader(data.compression_method, data.crc32, crypto_reader)?,
        })
    }

    /// Unwrap and return the inner reader object
    ///
    /// The position of the reader is undefined.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Holds the AES information of a file in the zip archive
#[derive(Debug)]
#[cfg(feature = "aes-crypto")]
pub struct AesInfo {
    /// The AES encryption mode
    pub aes_mode: AesMode,
    /// The verification key
    pub verification_value: [u8; PWD_VERIFY_LENGTH],
    /// The salt
    pub salt: Vec<u8>,
}

const fn unsupported_zip_error<T>(detail: &'static str) -> ZipResult<T> {
    Err(ZipError::UnsupportedArchive(detail))
}

/// Parse a central directory entry to collect the information for the file.
pub(crate) fn central_header_to_zip_file<R: Read + Seek>(
    reader: &mut R,
    central_directory: &CentralDirectoryInfo,
) -> ZipResult<ZipFileData> {
    let central_header_start = reader.stream_position()?;

    // Parse central header
    let block = ZipCentralEntryBlock::parse(reader)?;

    let file = central_header_to_zip_file_inner(
        reader,
        central_directory.archive_offset,
        central_header_start,
        block,
    )?;

    let central_header_end = reader.stream_position()?;

    if file.header_start >= central_directory.directory_start {
        return Err(InvalidArchive(
            "A local file entry can't start after the central directory",
        ));
    }

    let data_start = find_data_start(&file, reader)?;

    if data_start > central_directory.directory_start {
        return Err(InvalidArchive(
            "File data can't start after the central directory",
        ));
    }

    reader.seek(SeekFrom::Start(central_header_end))?;
    Ok(file)
}

#[inline]
fn read_variable_length_byte_field<R: Read>(reader: &mut R, len: usize) -> io::Result<Box<[u8]>> {
    let mut data = vec![0; len].into_boxed_slice();
    reader.read_exact(&mut data)?;
    Ok(data)
}

/// Parse a central directory entry to collect the information for the file.
fn central_header_to_zip_file_inner<R: Read>(
    reader: &mut R,
    archive_offset: u64,
    central_header_start: u64,
    block: ZipCentralEntryBlock,
) -> ZipResult<ZipFileData> {
    let ZipCentralEntryBlock {
        // magic,
        version_made_by,
        // version_to_extract,
        flags,
        compression_method,
        last_mod_time,
        last_mod_date,
        crc32,
        compressed_size,
        uncompressed_size,
        file_name_length,
        extra_field_length,
        file_comment_length,
        // disk_number,
        // internal_file_attributes,
        external_file_attributes,
        offset,
        ..
    } = block;

    let encrypted = flags & 1 == 1;
    let is_utf8 = flags & (1 << 11) != 0;
    let using_data_descriptor = flags & (1 << 3) != 0;

    let file_name_raw = read_variable_length_byte_field(reader, file_name_length as usize)?;
    let extra_field = read_variable_length_byte_field(reader, extra_field_length as usize)?;
    let file_comment_raw = read_variable_length_byte_field(reader, file_comment_length as usize)?;
    let file_name: Box<str> = match is_utf8 {
        true => String::from_utf8_lossy(&file_name_raw).into(),
        false => file_name_raw.clone().from_cp437(),
    };
    let file_comment: Box<str> = match is_utf8 {
        true => String::from_utf8_lossy(&file_comment_raw).into(),
        false => file_comment_raw.from_cp437(),
    };

    // Construct the result
    let mut result = ZipFileData {
        system: System::from((version_made_by >> 8) as u8),
        /* NB: this strips the top 8 bits! */
        version_made_by: version_made_by as u8,
        encrypted,
        using_data_descriptor,
        is_utf8,
        compression_method: CompressionMethod::parse_from_u16(compression_method),
        compression_level: None,
        last_modified_time: DateTime::try_from_msdos(last_mod_date, last_mod_time).ok(),
        crc32,
        compressed_size: compressed_size.into(),
        uncompressed_size: uncompressed_size.into(),
        file_name,
        file_name_raw,
        extra_field: Some(Arc::new(extra_field.to_vec())),
        central_extra_field: None,
        file_comment,
        header_start: offset.into(),
        extra_data_start: None,
        central_header_start,
        data_start: OnceLock::new(),
        external_attributes: external_file_attributes,
        large_file: false,
        aes_mode: None,
        aes_extra_data_start: 0,
        extra_fields: Vec::new(),
    };
    match parse_extra_field(&mut result) {
        Ok(stripped_extra_field) => {
            result.extra_field = stripped_extra_field;
        }
        Err(ZipError::Io(..)) => {}
        Err(e) => return Err(e),
    }

    let aes_enabled = result.compression_method == CompressionMethod::AES;
    if aes_enabled && result.aes_mode.is_none() {
        return Err(InvalidArchive(
            "AES encryption without AES extra data field",
        ));
    }

    // Account for shifted zip offsets.
    result.header_start = result
        .header_start
        .checked_add(archive_offset)
        .ok_or(InvalidArchive("Archive header is too large"))?;

    Ok(result)
}

pub(crate) fn parse_extra_field(file: &mut ZipFileData) -> ZipResult<Option<Arc<Vec<u8>>>> {
    let Some(ref extra_field) = file.extra_field else {
        return Ok(None);
    };
    let extra_field = extra_field.clone();
    let mut processed_extra_field = extra_field.clone();
    let len = extra_field.len();
    let mut reader = io::Cursor::new(&**extra_field);

    /* TODO: codify this structure into Zip64ExtraFieldBlock fields! */
    let mut position = reader.position() as usize;
    while (position) < len {
        let old_position = position;
        let remove = parse_single_extra_field(file, &mut reader, position as u64, false)?;
        position = reader.position() as usize;
        if remove {
            let remaining = len - (position - old_position);
            if remaining == 0 {
                return Ok(None);
            }
            let mut new_extra_field = Vec::with_capacity(remaining);
            new_extra_field.extend_from_slice(&extra_field[0..old_position]);
            new_extra_field.extend_from_slice(&extra_field[position..]);
            processed_extra_field = Arc::new(new_extra_field);
        }
    }
    Ok(Some(processed_extra_field))
}

pub(crate) fn parse_single_extra_field<R: Read>(
    file: &mut ZipFileData,
    reader: &mut R,
    bytes_already_read: u64,
    disallow_zip64: bool,
) -> ZipResult<bool> {
    let kind = reader.read_u16_le()?;
    let len = reader.read_u16_le()?;
    match kind {
        // Zip64 extended information extra field
        0x0001 => {
            if disallow_zip64 {
                return Err(InvalidArchive(
                    "Can't write a custom field using the ZIP64 ID",
                ));
            }
            file.large_file = true;
            let mut consumed_len = 0;
            if len >= 24 || file.uncompressed_size == spec::ZIP64_BYTES_THR {
                file.uncompressed_size = reader.read_u64_le()?;
                consumed_len += size_of::<u64>();
            }
            if len >= 24 || file.compressed_size == spec::ZIP64_BYTES_THR {
                file.compressed_size = reader.read_u64_le()?;
                consumed_len += size_of::<u64>();
            }
            if len >= 24 || file.header_start == spec::ZIP64_BYTES_THR {
                file.header_start = reader.read_u64_le()?;
                consumed_len += size_of::<u64>();
            }
            let Some(leftover_len) = (len as usize).checked_sub(consumed_len) else {
                return Err(InvalidArchive("ZIP64 extra-data field is the wrong length"));
            };
            reader.read_exact(&mut vec![0u8; leftover_len])?;
            return Ok(true);
        }
        0x9901 => {
            // AES
            if len != 7 {
                return Err(ZipError::UnsupportedArchive(
                    "AES extra data field has an unsupported length",
                ));
            }
            let vendor_version = reader.read_u16_le()?;
            let vendor_id = reader.read_u16_le()?;
            let mut out = [0u8];
            reader.read_exact(&mut out)?;
            let aes_mode = out[0];
            let compression_method = CompressionMethod::parse_from_u16(reader.read_u16_le()?);

            if vendor_id != 0x4541 {
                return Err(InvalidArchive("Invalid AES vendor"));
            }
            let vendor_version = match vendor_version {
                0x0001 => AesVendorVersion::Ae1,
                0x0002 => AesVendorVersion::Ae2,
                _ => return Err(InvalidArchive("Invalid AES vendor version")),
            };
            match aes_mode {
                0x01 => file.aes_mode = Some((AesMode::Aes128, vendor_version, compression_method)),
                0x02 => file.aes_mode = Some((AesMode::Aes192, vendor_version, compression_method)),
                0x03 => file.aes_mode = Some((AesMode::Aes256, vendor_version, compression_method)),
                _ => return Err(InvalidArchive("Invalid AES encryption strength")),
            };
            file.compression_method = compression_method;
            file.aes_extra_data_start = bytes_already_read;
        }
        0x5455 => {
            // extended timestamp
            // https://libzip.org/specifications/extrafld.txt

            file.extra_fields.push(ExtraField::ExtendedTimestamp(
                ExtendedTimestamp::try_from_reader(reader, len)?,
            ));
        }
        0x6375 => {
            // Info-ZIP Unicode Comment Extra Field
            // APPNOTE 4.6.8 and https://libzip.org/specifications/extrafld.txt
            file.file_comment = String::from_utf8(
                UnicodeExtraField::try_from_reader(reader, len)?
                    .unwrap_valid(file.file_comment.as_bytes())?
                    .into_vec(),
            )?
            .into();
        }
        0x7075 => {
            // Info-ZIP Unicode Path Extra Field
            // APPNOTE 4.6.9 and https://libzip.org/specifications/extrafld.txt
            file.file_name_raw = UnicodeExtraField::try_from_reader(reader, len)?
                .unwrap_valid(&file.file_name_raw)?;
            file.file_name =
                String::from_utf8(file.file_name_raw.clone().into_vec())?.into_boxed_str();
            file.is_utf8 = true;
        }
        _ => {
            reader.read_exact(&mut vec![0u8; len as usize])?;
            // Other fields are ignored
        }
    }
    Ok(false)
}

/// A trait for exposing file metadata inside the zip.
pub trait HasZipMetadata {
    /// Get the file metadata
    fn get_metadata(&self) -> &ZipFileData;
}

/// Methods for retrieving information on zip files
impl<'a> ZipFile<'a> {
    pub(crate) fn take_raw_reader(&mut self) -> io::Result<io::Take<&'a mut dyn Read>> {
        mem::replace(&mut self.reader, ZipFileReader::NoReader).into_inner()
    }

    /// Get the version of the file
    pub fn version_made_by(&self) -> (u8, u8) {
        (
            self.get_metadata().version_made_by / 10,
            self.get_metadata().version_made_by % 10,
        )
    }

    /// Get the name of the file
    ///
    /// # Warnings
    ///
    /// It is dangerous to use this name directly when extracting an archive.
    /// It may contain an absolute path (`/etc/shadow`), or break out of the
    /// current directory (`../runtime`). Carelessly writing to these paths
    /// allows an attacker to craft a ZIP archive that will overwrite critical
    /// files.
    ///
    /// You can use the [`ZipFile::enclosed_name`] method to validate the name
    /// as a safe path.
    pub fn name(&self) -> &str {
        &self.get_metadata().file_name
    }

    /// Get the name of the file, in the raw (internal) byte representation.
    ///
    /// The encoding of this data is currently undefined.
    pub fn name_raw(&self) -> &[u8] {
        &self.get_metadata().file_name_raw
    }

    /// Get the name of the file in a sanitized form. It truncates the name to the first NULL byte,
    /// removes a leading '/' and removes '..' parts.
    #[deprecated(
        since = "0.5.7",
        note = "by stripping `..`s from the path, the meaning of paths can change.
                `mangled_name` can be used if this behaviour is desirable"
    )]
    pub fn sanitized_name(&self) -> PathBuf {
        self.mangled_name()
    }

    /// Rewrite the path, ignoring any path components with special meaning.
    ///
    /// - Absolute paths are made relative
    /// - [`ParentDir`]s are ignored
    /// - Truncates the filename at a NULL byte
    ///
    /// This is appropriate if you need to be able to extract *something* from
    /// any archive, but will easily misrepresent trivial paths like
    /// `foo/../bar` as `foo/bar` (instead of `bar`). Because of this,
    /// [`ZipFile::enclosed_name`] is the better option in most scenarios.
    ///
    /// [`ParentDir`]: `PathBuf::Component::ParentDir`
    pub fn mangled_name(&self) -> PathBuf {
        self.get_metadata().file_name_sanitized()
    }

    /// Ensure the file path is safe to use as a [`Path`].
    ///
    /// - It can't contain NULL bytes
    /// - It can't resolve to a path outside the current directory
    ///   > `foo/../bar` is fine, `foo/../../bar` is not.
    /// - It can't be an absolute path
    ///
    /// This will read well-formed ZIP files correctly, and is resistant
    /// to path-based exploits. It is recommended over
    /// [`ZipFile::mangled_name`].
    pub fn enclosed_name(&self) -> Option<PathBuf> {
        self.get_metadata().enclosed_name()
    }

    /// Get the comment of the file
    pub fn comment(&self) -> &str {
        &self.get_metadata().file_comment
    }

    /// Get the compression method used to store the file
    pub fn compression(&self) -> CompressionMethod {
        self.get_metadata().compression_method
    }

    /// Get if the files is encrypted or not
    pub fn encrypted(&self) -> bool {
        self.data.encrypted
    }

    /// Get the size of the file, in bytes, in the archive
    pub fn compressed_size(&self) -> u64 {
        self.get_metadata().compressed_size
    }

    /// Get the size of the file, in bytes, when uncompressed
    pub fn size(&self) -> u64 {
        self.get_metadata().uncompressed_size
    }

    /// Get the time the file was last modified
    pub fn last_modified(&self) -> Option<DateTime> {
        self.data.last_modified_time
    }
    /// Returns whether the file is actually a directory
    pub fn is_dir(&self) -> bool {
        is_dir(self.name())
    }

    /// Returns whether the file is actually a symbolic link
    pub fn is_symlink(&self) -> bool {
        self.unix_mode()
            .is_some_and(|mode| mode & S_IFLNK == S_IFLNK)
    }

    /// Returns whether the file is a normal file (i.e. not a directory or symlink)
    pub fn is_file(&self) -> bool {
        !self.is_dir() && !self.is_symlink()
    }

    /// Get unix mode for the file
    pub fn unix_mode(&self) -> Option<u32> {
        self.get_metadata().unix_mode()
    }

    /// Get the CRC32 hash of the original file
    pub fn crc32(&self) -> u32 {
        self.get_metadata().crc32
    }

    /// Get the extra data of the zip header for this file
    pub fn extra_data(&self) -> Option<&[u8]> {
        self.get_metadata()
            .extra_field
            .as_ref()
            .map(|v| v.deref().deref())
    }

    /// Get the starting offset of the data of the compressed file
    pub fn data_start(&self) -> u64 {
        *self.data.data_start.get().unwrap()
    }

    /// Get the starting offset of the zip header for this file
    pub fn header_start(&self) -> u64 {
        self.get_metadata().header_start
    }
    /// Get the starting offset of the zip header in the central directory for this file
    pub fn central_header_start(&self) -> u64 {
        self.get_metadata().central_header_start
    }
}

/// Methods for retrieving information on zip files
impl ZipFile<'_> {
    /// iterate through all extra fields
    pub fn extra_data_fields(&self) -> impl Iterator<Item = &ExtraField> {
        self.data.extra_fields.iter()
    }
}

impl HasZipMetadata for ZipFile<'_> {
    fn get_metadata(&self) -> &ZipFileData {
        self.data.as_ref()
    }
}

impl Read for ZipFile<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.reader.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.reader.read_to_string(buf)
    }
}

impl<R: Read> Read for ZipFileSeek<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.reader {
            ZipFileSeekReader::Raw(r) => r.read(buf),
        }
    }
}

impl<R: Seek> Seek for ZipFileSeek<'_, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match &mut self.reader {
            ZipFileSeekReader::Raw(r) => r.seek(pos),
        }
    }
}

impl<R> HasZipMetadata for ZipFileSeek<'_, R> {
    fn get_metadata(&self) -> &ZipFileData {
        self.data.as_ref()
    }
}

impl Drop for ZipFile<'_> {
    fn drop(&mut self) {
        // self.data is Owned, this reader is constructed by a streaming reader.
        // In this case, we want to exhaust the reader so that the next file is accessible.
        if let Cow::Owned(_) = self.data {
            // Get the inner `Take` reader so all decryption, decompression and CRC calculation is skipped.
            if let Ok(mut inner) = self.take_raw_reader() {
                let _ = copy(&mut inner, &mut sink());
            }
        }
    }
}

/// Read ZipFile structures from a non-seekable reader.
///
/// This is an alternative method to read a zip file. If possible, use the ZipArchive functions
/// as some information will be missing when reading this manner.
///
/// Reads a file header from the start of the stream. Will return `Ok(Some(..))` if a file is
/// present at the start of the stream. Returns `Ok(None)` if the start of the central directory
/// is encountered. No more files should be read after this.
///
/// The Drop implementation of ZipFile ensures that the reader will be correctly positioned after
/// the structure is done.
///
/// Missing fields are:
/// * `comment`: set to an empty string
/// * `data_start`: set to 0
/// * `external_attributes`: `unix_mode()`: will return None
pub fn read_zipfile_from_stream<R: Read>(reader: &mut R) -> ZipResult<Option<ZipFile<'_>>> {
    // We can't use the typical ::parse() method, as we follow separate code paths depending on the
    // "magic" value (since the magic value will be from the central directory header if we've
    // finished iterating over all the actual files).
    /* TODO: smallvec? */

    let mut block = ZipLocalEntryBlock::zeroed();
    reader.read_exact(block.as_bytes_mut())?;

    match block.magic().from_le() {
        spec::Magic::LOCAL_FILE_HEADER_SIGNATURE => (),
        spec::Magic::CENTRAL_DIRECTORY_HEADER_SIGNATURE => return Ok(None),
        _ => return Err(ZipLocalEntryBlock::WRONG_MAGIC_ERROR),
    }

    let block = block.from_le();

    let mut result = ZipFileData::from_local_block(block, reader)?;

    match parse_extra_field(&mut result) {
        Ok(..) | Err(ZipError::Io(..)) => {}
        Err(e) => return Err(e),
    }

    let limit_reader = (reader as &mut dyn Read).take(result.compressed_size);

    let result_crc32 = result.crc32;
    let result_compression_method = result.compression_method;
    let crypto_reader = make_crypto_reader(&result, limit_reader, None, None)?;

    Ok(Some(ZipFile {
        data: Cow::Owned(result),
        reader: make_reader(result_compression_method, result_crc32, crypto_reader)?,
    }))
}

#[cfg(test)]
mod test {
    use crate::result::ZipResult;
    use crate::write::SimpleFileOptions;
    use crate::CompressionMethod::Stored;
    use crate::{ZipArchive, ZipWriter};
    use std::io::{Cursor, Read, Write};
    use tempfile::TempDir;

    #[test]
    fn invalid_offset() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/invalid_offset.zip"));
        let reader = ZipArchive::new(Cursor::new(v));
        assert!(reader.is_err());
    }

    #[test]
    fn invalid_offset2() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/invalid_offset2.zip"));
        let reader = ZipArchive::new(Cursor::new(v));
        assert!(reader.is_err());
    }

    #[test]
    fn zip64_with_leading_junk() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/zip64_demo.zip"));
        let reader = ZipArchive::new(Cursor::new(v)).unwrap();
        assert_eq!(reader.len(), 1);
    }

    #[test]
    fn zip_contents() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/mimetype.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        assert_eq!(reader.comment(), b"");
        assert_eq!(reader.by_index(0).unwrap().central_header_start(), 77);
    }

    #[test]
    fn zip_read_streaming() {
        use super::read_zipfile_from_stream;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/mimetype.zip"));
        let mut reader = Cursor::new(v);
        loop {
            if read_zipfile_from_stream(&mut reader).unwrap().is_none() {
                break;
            }
        }
    }

    #[test]
    fn zip_clone() {
        use super::ZipArchive;
        use std::io::Read;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/mimetype.zip"));
        let mut reader1 = ZipArchive::new(Cursor::new(v)).unwrap();
        let mut reader2 = reader1.clone();

        let mut file1 = reader1.by_index(0).unwrap();
        let mut file2 = reader2.by_index(0).unwrap();

        let t = file1.last_modified().unwrap();
        assert_eq!(
            (
                t.year(),
                t.month(),
                t.day(),
                t.hour(),
                t.minute(),
                t.second()
            ),
            (1980, 1, 1, 0, 0, 0)
        );

        let mut buf1 = [0; 5];
        let mut buf2 = [0; 5];
        let mut buf3 = [0; 5];
        let mut buf4 = [0; 5];

        file1.read_exact(&mut buf1).unwrap();
        file2.read_exact(&mut buf2).unwrap();
        file1.read_exact(&mut buf3).unwrap();
        file2.read_exact(&mut buf4).unwrap();

        assert_eq!(buf1, buf2);
        assert_eq!(buf3, buf4);
        assert_ne!(buf1, buf3);
    }

    #[test]
    fn file_and_dir_predicates() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/files_and_dirs.zip"));
        let mut zip = ZipArchive::new(Cursor::new(v)).unwrap();

        for i in 0..zip.len() {
            let zip_file = zip.by_index(i).unwrap();
            let full_name = zip_file.enclosed_name().unwrap();
            let file_name = full_name.file_name().unwrap().to_str().unwrap();
            assert!(
                (file_name.starts_with("dir") && zip_file.is_dir())
                    || (file_name.starts_with("file") && zip_file.is_file())
            );
        }
    }

    #[test]
    fn zip64_magic_in_filenames() {
        let files = vec![
            include_bytes!("../tests/data/zip64_magic_in_filename_1.zip").to_vec(),
            include_bytes!("../tests/data/zip64_magic_in_filename_2.zip").to_vec(),
            include_bytes!("../tests/data/zip64_magic_in_filename_3.zip").to_vec(),
            include_bytes!("../tests/data/zip64_magic_in_filename_4.zip").to_vec(),
            include_bytes!("../tests/data/zip64_magic_in_filename_5.zip").to_vec(),
        ];
        // Although we don't allow adding files whose names contain the ZIP64 CDB-end or
        // CDB-end-locator signatures, we still read them when they aren't genuinely ambiguous.
        for file in files {
            ZipArchive::new(Cursor::new(file)).unwrap();
        }
    }

    /// test case to ensure we don't preemptively over allocate based on the
    /// declared number of files in the CDE of an invalid zip when the number of
    /// files declared is more than the alleged offset in the CDE
    #[test]
    fn invalid_cde_number_of_files_allocation_smaller_offset() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!(
            "../tests/data/invalid_cde_number_of_files_allocation_smaller_offset.zip"
        ));
        let reader = ZipArchive::new(Cursor::new(v));
        assert!(reader.is_err() || reader.unwrap().is_empty());
    }

    /// test case to ensure we don't preemptively over allocate based on the
    /// declared number of files in the CDE of an invalid zip when the number of
    /// files declared is less than the alleged offset in the CDE
    #[test]
    fn invalid_cde_number_of_files_allocation_greater_offset() {
        use super::ZipArchive;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!(
            "../tests/data/invalid_cde_number_of_files_allocation_greater_offset.zip"
        ));
        let reader = ZipArchive::new(Cursor::new(v));
        assert!(reader.is_err());
    }

    #[cfg(feature = "deflate64")]
    #[test]
    fn deflate64_index_out_of_bounds() -> std::io::Result<()> {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!(
            "../tests/data/raw_deflate64_index_out_of_bounds.zip"
        ));
        let mut reader = ZipArchive::new(Cursor::new(v))?;
        std::io::copy(&mut reader.by_index(0)?, &mut std::io::sink()).expect_err("Invalid file");
        Ok(())
    }

    #[cfg(feature = "deflate64")]
    #[test]
    fn deflate64_not_enough_space() {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/deflate64_issue_25.zip"));
        ZipArchive::new(Cursor::new(v)).expect_err("Invalid file");
    }

    #[cfg(feature = "_deflate-any")]
    #[test]
    fn test_read_with_data_descriptor() {
        use std::io::Read;

        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/data_descriptor.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        let mut decompressed = [0u8; 16];
        let mut file = reader.by_index(0).unwrap();
        assert_eq!(file.read(&mut decompressed).unwrap(), 12);
    }

    #[test]
    fn test_is_symlink() -> std::io::Result<()> {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/symlink.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        assert!(reader.by_index(0).unwrap().is_symlink());
        let tempdir = TempDir::with_prefix("test_is_symlink")?;
        reader.extract(&tempdir).unwrap();
        assert!(tempdir.path().join("bar").is_symlink());
        Ok(())
    }

    #[test]
    #[cfg(feature = "_deflate-any")]
    fn test_utf8_extra_field() {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/chinese.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        reader.by_name("七个房间.txt").unwrap();
    }

    #[test]
    fn test_utf8() {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/linux-7z.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        reader.by_name("你好.txt").unwrap();
    }

    #[test]
    fn test_utf8_2() {
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/windows-7zip.zip"));
        let mut reader = ZipArchive::new(Cursor::new(v)).unwrap();
        reader.by_name("你好.txt").unwrap();
    }

    #[test]
    fn test_64k_files() -> ZipResult<()> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions {
            compression_method: Stored,
            ..Default::default()
        };
        for i in 0..=u16::MAX {
            let file_name = format!("{i}.txt");
            writer.start_file(&*file_name, options)?;
            writer.write_all(i.to_string().as_bytes())?;
        }

        let mut reader = ZipArchive::new(writer.finish()?)?;
        for i in 0..=u16::MAX {
            let expected_name = format!("{i}.txt");
            let expected_contents = i.to_string();
            let expected_contents = expected_contents.as_bytes();
            let mut file = reader.by_name(&expected_name)?;
            let mut contents = Vec::with_capacity(expected_contents.len());
            file.read_to_end(&mut contents)?;
            assert_eq!(contents, expected_contents);
            drop(file);
            contents.clear();
            let mut file = reader.by_index(i as usize)?;
            file.read_to_end(&mut contents)?;
            assert_eq!(contents, expected_contents);
        }
        Ok(())
    }
}
