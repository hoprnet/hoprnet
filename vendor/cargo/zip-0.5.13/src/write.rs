//! Types for creating ZIP archives

use crate::compression::CompressionMethod;
use crate::read::{central_header_to_zip_file, ZipArchive, ZipFile};
use crate::result::{ZipError, ZipResult};
use crate::spec;
use crate::types::{DateTime, System, ZipFileData, DEFAULT_VERSION};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc32fast::Hasher;
use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::mem;

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
))]
use flate2::write::DeflateEncoder;

#[cfg(feature = "bzip2")]
use bzip2::write::BzEncoder;

enum GenericZipWriter<W: Write + io::Seek> {
    Closed,
    Storer(W),
    #[cfg(any(
        feature = "deflate",
        feature = "deflate-miniz",
        feature = "deflate-zlib"
    ))]
    Deflater(DeflateEncoder<W>),
    #[cfg(feature = "bzip2")]
    Bzip2(BzEncoder<W>),
}

/// ZIP archive generator
///
/// Handles the bookkeeping involved in building an archive, and provides an
/// API to edit its contents.
///
/// ```
/// # fn doit() -> zip::result::ZipResult<()>
/// # {
/// # use zip::ZipWriter;
/// use std::io::Write;
/// use zip::write::FileOptions;
///
/// // We use a buffer here, though you'd normally use a `File`
/// let mut buf = [0; 65536];
/// let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf[..]));
///
/// let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
/// zip.start_file("hello_world.txt", options)?;
/// zip.write(b"Hello, World!")?;
///
/// // Apply the changes you've made.
/// // Dropping the `ZipWriter` will have the same effect, but may silently fail
/// zip.finish()?;
///
/// # Ok(())
/// # }
/// # doit().unwrap();
/// ```
pub struct ZipWriter<W: Write + io::Seek> {
    inner: GenericZipWriter<W>,
    files: Vec<ZipFileData>,
    stats: ZipWriterStats,
    writing_to_file: bool,
    writing_to_extra_field: bool,
    writing_to_central_extra_field_only: bool,
    writing_raw: bool,
    comment: Vec<u8>,
}

#[derive(Default)]
struct ZipWriterStats {
    hasher: Hasher,
    start: u64,
    bytes_written: u64,
}

struct ZipRawValues {
    crc32: u32,
    compressed_size: u64,
    uncompressed_size: u64,
}

/// Metadata for a file to be written
#[derive(Copy, Clone)]
pub struct FileOptions {
    compression_method: CompressionMethod,
    last_modified_time: DateTime,
    permissions: Option<u32>,
    large_file: bool,
}

impl FileOptions {
    /// Construct a new FileOptions object
    pub fn default() -> FileOptions {
        FileOptions {
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            compression_method: CompressionMethod::Deflated,
            #[cfg(not(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            )))]
            compression_method: CompressionMethod::Stored,
            #[cfg(feature = "time")]
            last_modified_time: DateTime::from_time(time::now()).unwrap_or_default(),
            #[cfg(not(feature = "time"))]
            last_modified_time: DateTime::default(),
            permissions: None,
            large_file: false,
        }
    }

    /// Set the compression method for the new file
    ///
    /// The default is `CompressionMethod::Deflated`. If the deflate compression feature is
    /// disabled, `CompressionMethod::Stored` becomes the default.
    pub fn compression_method(mut self, method: CompressionMethod) -> FileOptions {
        self.compression_method = method;
        self
    }

    /// Set the last modified time
    ///
    /// The default is the current timestamp if the 'time' feature is enabled, and 1980-01-01
    /// otherwise
    pub fn last_modified_time(mut self, mod_time: DateTime) -> FileOptions {
        self.last_modified_time = mod_time;
        self
    }

    /// Set the permissions for the new file.
    ///
    /// The format is represented with unix-style permissions.
    /// The default is `0o644`, which represents `rw-r--r--` for files,
    /// and `0o755`, which represents `rwxr-xr-x` for directories
    pub fn unix_permissions(mut self, mode: u32) -> FileOptions {
        self.permissions = Some(mode & 0o777);
        self
    }

    /// Set whether the new file's compressed and uncompressed size is less than 4 GiB.
    ///
    /// If set to `false` and the file exceeds the limit, an I/O error is thrown. If set to `true`,
    /// readers will require ZIP64 support and if the file does not exceed the limit, 20 B are
    /// wasted. The default is `false`.
    pub fn large_file(mut self, large: bool) -> FileOptions {
        self.large_file = large;
        self
    }
}

impl Default for FileOptions {
    fn default() -> Self {
        Self::default()
    }
}

impl<W: Write + io::Seek> Write for ZipWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.writing_to_file {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No file has been started",
            ));
        }
        match self.inner.ref_mut() {
            Some(ref mut w) => {
                if self.writing_to_extra_field {
                    self.files.last_mut().unwrap().extra_field.write(buf)
                } else {
                    let write_result = w.write(buf);
                    if let Ok(count) = write_result {
                        self.stats.update(&buf[0..count]);
                        if self.stats.bytes_written > 0xFFFFFFFF
                            && !self.files.last_mut().unwrap().large_file
                        {
                            let _inner = mem::replace(&mut self.inner, GenericZipWriter::Closed);
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "Large file option has not been set",
                            ));
                        }
                    }
                    write_result
                }
            }
            None => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "ZipWriter was already closed",
            )),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner.ref_mut() {
            Some(ref mut w) => w.flush(),
            None => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "ZipWriter was already closed",
            )),
        }
    }
}

impl ZipWriterStats {
    fn update(&mut self, buf: &[u8]) {
        self.hasher.update(buf);
        self.bytes_written += buf.len() as u64;
    }
}

impl<A: Read + Write + io::Seek> ZipWriter<A> {
    /// Initializes the archive from an existing ZIP archive, making it ready for append.
    pub fn new_append(mut readwriter: A) -> ZipResult<ZipWriter<A>> {
        let (footer, cde_start_pos) = spec::CentralDirectoryEnd::find_and_parse(&mut readwriter)?;

        if footer.disk_number != footer.disk_with_central_directory {
            return Err(ZipError::UnsupportedArchive(
                "Support for multi-disk files is not implemented",
            ));
        }

        let (archive_offset, directory_start, number_of_files) =
            ZipArchive::get_directory_counts(&mut readwriter, &footer, cde_start_pos)?;

        if let Err(_) = readwriter.seek(io::SeekFrom::Start(directory_start)) {
            return Err(ZipError::InvalidArchive(
                "Could not seek to start of central directory",
            ));
        }

        let files = (0..number_of_files)
            .map(|_| central_header_to_zip_file(&mut readwriter, archive_offset))
            .collect::<Result<Vec<_>, _>>()?;

        let _ = readwriter.seek(io::SeekFrom::Start(directory_start)); // seek directory_start to overwrite it

        Ok(ZipWriter {
            inner: GenericZipWriter::Storer(readwriter),
            files,
            stats: Default::default(),
            writing_to_file: false,
            writing_to_extra_field: false,
            writing_to_central_extra_field_only: false,
            comment: footer.zip_file_comment,
            writing_raw: true, // avoid recomputing the last file's header
        })
    }
}

impl<W: Write + io::Seek> ZipWriter<W> {
    /// Initializes the archive.
    ///
    /// Before writing to this object, the [`ZipWriter::start_file`] function should be called.
    pub fn new(inner: W) -> ZipWriter<W> {
        ZipWriter {
            inner: GenericZipWriter::Storer(inner),
            files: Vec::new(),
            stats: Default::default(),
            writing_to_file: false,
            writing_to_extra_field: false,
            writing_to_central_extra_field_only: false,
            writing_raw: false,
            comment: Vec::new(),
        }
    }

    /// Set ZIP archive comment.
    pub fn set_comment<S>(&mut self, comment: S)
    where
        S: Into<String>,
    {
        self.set_raw_comment(comment.into().into())
    }

    /// Set ZIP archive comment.
    ///
    /// This sets the raw bytes of the comment. The comment
    /// is typically expected to be encoded in UTF-8
    pub fn set_raw_comment(&mut self, comment: Vec<u8>) {
        self.comment = comment;
    }

    /// Start a new file for with the requested options.
    fn start_entry<S>(
        &mut self,
        name: S,
        options: FileOptions,
        raw_values: Option<ZipRawValues>,
    ) -> ZipResult<()>
    where
        S: Into<String>,
    {
        self.finish_file()?;

        let raw_values = raw_values.unwrap_or_else(|| ZipRawValues {
            crc32: 0,
            compressed_size: 0,
            uncompressed_size: 0,
        });

        {
            let writer = self.inner.get_plain();
            let header_start = writer.seek(io::SeekFrom::Current(0))?;

            let permissions = options.permissions.unwrap_or(0o100644);
            let mut file = ZipFileData {
                system: System::Unix,
                version_made_by: DEFAULT_VERSION,
                encrypted: false,
                using_data_descriptor: false,
                compression_method: options.compression_method,
                last_modified_time: options.last_modified_time,
                crc32: raw_values.crc32,
                compressed_size: raw_values.compressed_size,
                uncompressed_size: raw_values.uncompressed_size,
                file_name: name.into(),
                file_name_raw: Vec::new(), // Never used for saving
                extra_field: Vec::new(),
                file_comment: String::new(),
                header_start,
                data_start: 0,
                central_header_start: 0,
                external_attributes: permissions << 16,
                large_file: options.large_file,
            };
            write_local_file_header(writer, &file)?;

            let header_end = writer.seek(io::SeekFrom::Current(0))?;
            self.stats.start = header_end;
            file.data_start = header_end;

            self.stats.bytes_written = 0;
            self.stats.hasher = Hasher::new();

            self.files.push(file);
        }

        Ok(())
    }

    fn finish_file(&mut self) -> ZipResult<()> {
        if self.writing_to_extra_field {
            // Implicitly calling [`ZipWriter::end_extra_data`] for empty files.
            self.end_extra_data()?;
        }
        self.inner.switch_to(CompressionMethod::Stored)?;
        let writer = self.inner.get_plain();

        if !self.writing_raw {
            let file = match self.files.last_mut() {
                None => return Ok(()),
                Some(f) => f,
            };
            file.crc32 = self.stats.hasher.clone().finalize();
            file.uncompressed_size = self.stats.bytes_written;

            let file_end = writer.seek(io::SeekFrom::Current(0))?;
            file.compressed_size = file_end - self.stats.start;

            update_local_file_header(writer, file)?;
            writer.seek(io::SeekFrom::Start(file_end))?;
        }

        self.writing_to_file = false;
        self.writing_raw = false;
        Ok(())
    }

    /// Create a file in the archive and start writing its' contents.
    ///
    /// The data should be written using the [`io::Write`] implementation on this [`ZipWriter`]
    pub fn start_file<S>(&mut self, name: S, mut options: FileOptions) -> ZipResult<()>
    where
        S: Into<String>,
    {
        if options.permissions.is_none() {
            options.permissions = Some(0o644);
        }
        *options.permissions.as_mut().unwrap() |= 0o100000;
        self.start_entry(name, options, None)?;
        self.inner.switch_to(options.compression_method)?;
        self.writing_to_file = true;
        Ok(())
    }

    /// Starts a file, taking a Path as argument.
    ///
    /// This function ensures that the '/' path separator is used. It also ignores all non 'Normal'
    /// Components, such as a starting '/' or '..' and '.'.
    #[deprecated(
        since = "0.5.7",
        note = "by stripping `..`s from the path, the meaning of paths can change. Use `start_file` instead."
    )]
    pub fn start_file_from_path(
        &mut self,
        path: &std::path::Path,
        options: FileOptions,
    ) -> ZipResult<()> {
        self.start_file(path_to_string(path), options)
    }

    /// Create an aligned file in the archive and start writing its' contents.
    ///
    /// Returns the number of padding bytes required to align the file.
    ///
    /// The data should be written using the [`io::Write`] implementation on this [`ZipWriter`]
    pub fn start_file_aligned<S>(
        &mut self,
        name: S,
        options: FileOptions,
        align: u16,
    ) -> Result<u64, ZipError>
    where
        S: Into<String>,
    {
        let data_start = self.start_file_with_extra_data(name, options)?;
        let align = align as u64;
        if align > 1 && data_start % align != 0 {
            let pad_length = (align - (data_start + 4) % align) % align;
            let pad = vec![0; pad_length as usize];
            self.write_all(b"za").map_err(ZipError::from)?; // 0x617a
            self.write_u16::<LittleEndian>(pad.len() as u16)
                .map_err(ZipError::from)?;
            self.write_all(&pad).map_err(ZipError::from)?;
            assert_eq!(self.end_local_start_central_extra_data()? % align, 0);
        }
        let extra_data_end = self.end_extra_data()?;
        Ok(extra_data_end - data_start)
    }

    /// Create a file in the archive and start writing its extra data first.
    ///
    /// Finish writing extra data and start writing file data with [`ZipWriter::end_extra_data`].
    /// Optionally, distinguish local from central extra data with
    /// [`ZipWriter::end_local_start_central_extra_data`].
    ///
    /// Returns the preliminary starting offset of the file data without any extra data allowing to
    /// align the file data by calculating a pad length to be prepended as part of the extra data.
    ///
    /// The data should be written using the [`io::Write`] implementation on this [`ZipWriter`]
    ///
    /// ```
    /// use byteorder::{LittleEndian, WriteBytesExt};
    /// use zip::{ZipArchive, ZipWriter, result::ZipResult};
    /// use zip::{write::FileOptions, CompressionMethod};
    /// use std::io::{Write, Cursor};
    ///
    /// # fn main() -> ZipResult<()> {
    /// let mut archive = Cursor::new(Vec::new());
    ///
    /// {
    ///     let mut zip = ZipWriter::new(&mut archive);
    ///     let options = FileOptions::default()
    ///         .compression_method(CompressionMethod::Stored);
    ///
    ///     zip.start_file_with_extra_data("identical_extra_data.txt", options)?;
    ///     let extra_data = b"local and central extra data";
    ///     zip.write_u16::<LittleEndian>(0xbeef)?;
    ///     zip.write_u16::<LittleEndian>(extra_data.len() as u16)?;
    ///     zip.write_all(extra_data)?;
    ///     zip.end_extra_data()?;
    ///     zip.write_all(b"file data")?;
    ///
    ///     let data_start = zip.start_file_with_extra_data("different_extra_data.txt", options)?;
    ///     let extra_data = b"local extra data";
    ///     zip.write_u16::<LittleEndian>(0xbeef)?;
    ///     zip.write_u16::<LittleEndian>(extra_data.len() as u16)?;
    ///     zip.write_all(extra_data)?;
    ///     let data_start = data_start as usize + 4 + extra_data.len() + 4;
    ///     let align = 64;
    ///     let pad_length = (align - data_start % align) % align;
    ///     assert_eq!(pad_length, 19);
    ///     zip.write_u16::<LittleEndian>(0xdead)?;
    ///     zip.write_u16::<LittleEndian>(pad_length as u16)?;
    ///     zip.write_all(&vec![0; pad_length])?;
    ///     let data_start = zip.end_local_start_central_extra_data()?;
    ///     assert_eq!(data_start as usize % align, 0);
    ///     let extra_data = b"central extra data";
    ///     zip.write_u16::<LittleEndian>(0xbeef)?;
    ///     zip.write_u16::<LittleEndian>(extra_data.len() as u16)?;
    ///     zip.write_all(extra_data)?;
    ///     zip.end_extra_data()?;
    ///     zip.write_all(b"file data")?;
    ///
    ///     zip.finish()?;
    /// }
    ///
    /// let mut zip = ZipArchive::new(archive)?;
    /// assert_eq!(&zip.by_index(0)?.extra_data()[4..], b"local and central extra data");
    /// assert_eq!(&zip.by_index(1)?.extra_data()[4..], b"central extra data");
    /// # Ok(())
    /// # }
    /// ```
    pub fn start_file_with_extra_data<S>(
        &mut self,
        name: S,
        mut options: FileOptions,
    ) -> ZipResult<u64>
    where
        S: Into<String>,
    {
        if options.permissions.is_none() {
            options.permissions = Some(0o644);
        }
        *options.permissions.as_mut().unwrap() |= 0o100000;
        self.start_entry(name, options, None)?;
        self.writing_to_file = true;
        self.writing_to_extra_field = true;
        Ok(self.files.last().unwrap().data_start)
    }

    /// End local and start central extra data. Requires [`ZipWriter::start_file_with_extra_data`].
    ///
    /// Returns the final starting offset of the file data.
    pub fn end_local_start_central_extra_data(&mut self) -> ZipResult<u64> {
        let data_start = self.end_extra_data()?;
        self.files.last_mut().unwrap().extra_field.clear();
        self.writing_to_extra_field = true;
        self.writing_to_central_extra_field_only = true;
        Ok(data_start)
    }

    /// End extra data and start file data. Requires [`ZipWriter::start_file_with_extra_data`].
    ///
    /// Returns the final starting offset of the file data.
    pub fn end_extra_data(&mut self) -> ZipResult<u64> {
        // Require `start_file_with_extra_data()`. Ensures `file` is some.
        if !self.writing_to_extra_field {
            return Err(ZipError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Not writing to extra field",
            )));
        }
        let file = self.files.last_mut().unwrap();

        validate_extra_data(&file)?;

        if !self.writing_to_central_extra_field_only {
            let writer = self.inner.get_plain();

            // Append extra data to local file header and keep it for central file header.
            writer.write_all(&file.extra_field)?;

            // Update final `data_start`.
            let header_end = file.data_start + file.extra_field.len() as u64;
            self.stats.start = header_end;
            file.data_start = header_end;

            // Update extra field length in local file header.
            let extra_field_length =
                if file.large_file { 20 } else { 0 } + file.extra_field.len() as u16;
            writer.seek(io::SeekFrom::Start(file.header_start + 28))?;
            writer.write_u16::<LittleEndian>(extra_field_length)?;
            writer.seek(io::SeekFrom::Start(header_end))?;

            self.inner.switch_to(file.compression_method)?;
        }

        self.writing_to_extra_field = false;
        self.writing_to_central_extra_field_only = false;
        Ok(file.data_start)
    }

    /// Add a new file using the already compressed data from a ZIP file being read and renames it, this
    /// allows faster copies of the `ZipFile` since there is no need to decompress and compress it again.
    /// Any `ZipFile` metadata is copied and not checked, for example the file CRC.

    /// ```no_run
    /// use std::fs::File;
    /// use std::io::{Read, Seek, Write};
    /// use zip::{ZipArchive, ZipWriter};
    ///
    /// fn copy_rename<R, W>(
    ///     src: &mut ZipArchive<R>,
    ///     dst: &mut ZipWriter<W>,
    /// ) -> zip::result::ZipResult<()>
    /// where
    ///     R: Read + Seek,
    ///     W: Write + Seek,
    /// {
    ///     // Retrieve file entry by name
    ///     let file = src.by_name("src_file.txt")?;
    ///
    ///     // Copy and rename the previously obtained file entry to the destination zip archive
    ///     dst.raw_copy_file_rename(file, "new_name.txt")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn raw_copy_file_rename<S>(&mut self, mut file: ZipFile, name: S) -> ZipResult<()>
    where
        S: Into<String>,
    {
        let options = FileOptions::default()
            .last_modified_time(file.last_modified())
            .compression_method(file.compression());
        if let Some(perms) = file.unix_mode() {
            options.unix_permissions(perms);
        }

        let raw_values = ZipRawValues {
            crc32: file.crc32(),
            compressed_size: file.compressed_size(),
            uncompressed_size: file.size(),
        };

        self.start_entry(name, options, Some(raw_values))?;
        self.writing_to_file = true;
        self.writing_raw = true;

        io::copy(file.get_raw_reader(), self)?;

        Ok(())
    }

    /// Add a new file using the already compressed data from a ZIP file being read, this allows faster
    /// copies of the `ZipFile` since there is no need to decompress and compress it again. Any `ZipFile`
    /// metadata is copied and not checked, for example the file CRC.
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::{Read, Seek, Write};
    /// use zip::{ZipArchive, ZipWriter};
    ///
    /// fn copy<R, W>(src: &mut ZipArchive<R>, dst: &mut ZipWriter<W>) -> zip::result::ZipResult<()>
    /// where
    ///     R: Read + Seek,
    ///     W: Write + Seek,
    /// {
    ///     // Retrieve file entry by name
    ///     let file = src.by_name("src_file.txt")?;
    ///
    ///     // Copy the previously obtained file entry to the destination zip archive
    ///     dst.raw_copy_file(file)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn raw_copy_file(&mut self, file: ZipFile) -> ZipResult<()> {
        let name = file.name().to_owned();
        self.raw_copy_file_rename(file, name)
    }

    /// Add a directory entry.
    ///
    /// You can't write data to the file afterwards.
    pub fn add_directory<S>(&mut self, name: S, mut options: FileOptions) -> ZipResult<()>
    where
        S: Into<String>,
    {
        if options.permissions.is_none() {
            options.permissions = Some(0o755);
        }
        *options.permissions.as_mut().unwrap() |= 0o40000;
        options.compression_method = CompressionMethod::Stored;

        let name_as_string = name.into();
        // Append a slash to the filename if it does not end with it.
        let name_with_slash = match name_as_string.chars().last() {
            Some('/') | Some('\\') => name_as_string,
            _ => name_as_string + "/",
        };

        self.start_entry(name_with_slash, options, None)?;
        self.writing_to_file = false;
        Ok(())
    }

    /// Add a directory entry, taking a Path as argument.
    ///
    /// This function ensures that the '/' path seperator is used. It also ignores all non 'Normal'
    /// Components, such as a starting '/' or '..' and '.'.
    #[deprecated(
        since = "0.5.7",
        note = "by stripping `..`s from the path, the meaning of paths can change. Use `add_directory` instead."
    )]
    pub fn add_directory_from_path(
        &mut self,
        path: &std::path::Path,
        options: FileOptions,
    ) -> ZipResult<()> {
        self.add_directory(path_to_string(path), options)
    }

    /// Finish the last file and write all other zip-structures
    ///
    /// This will return the writer, but one should normally not append any data to the end of the file.
    /// Note that the zipfile will also be finished on drop.
    pub fn finish(&mut self) -> ZipResult<W> {
        self.finalize()?;
        let inner = mem::replace(&mut self.inner, GenericZipWriter::Closed);
        Ok(inner.unwrap())
    }

    fn finalize(&mut self) -> ZipResult<()> {
        self.finish_file()?;

        {
            let writer = self.inner.get_plain();

            let central_start = writer.seek(io::SeekFrom::Current(0))?;
            for file in self.files.iter() {
                write_central_directory_header(writer, file)?;
            }
            let central_size = writer.seek(io::SeekFrom::Current(0))? - central_start;

            if self.files.len() > 0xFFFF || central_size > 0xFFFFFFFF || central_start > 0xFFFFFFFF
            {
                let zip64_footer = spec::Zip64CentralDirectoryEnd {
                    version_made_by: DEFAULT_VERSION as u16,
                    version_needed_to_extract: DEFAULT_VERSION as u16,
                    disk_number: 0,
                    disk_with_central_directory: 0,
                    number_of_files_on_this_disk: self.files.len() as u64,
                    number_of_files: self.files.len() as u64,
                    central_directory_size: central_size,
                    central_directory_offset: central_start,
                };

                zip64_footer.write(writer)?;

                let zip64_footer = spec::Zip64CentralDirectoryEndLocator {
                    disk_with_central_directory: 0,
                    end_of_central_directory_offset: central_start + central_size,
                    number_of_disks: 1,
                };

                zip64_footer.write(writer)?;
            }

            let number_of_files = if self.files.len() > 0xFFFF {
                0xFFFF
            } else {
                self.files.len() as u16
            };
            let footer = spec::CentralDirectoryEnd {
                disk_number: 0,
                disk_with_central_directory: 0,
                zip_file_comment: self.comment.clone(),
                number_of_files_on_this_disk: number_of_files,
                number_of_files,
                central_directory_size: if central_size > 0xFFFFFFFF {
                    0xFFFFFFFF
                } else {
                    central_size as u32
                },
                central_directory_offset: if central_start > 0xFFFFFFFF {
                    0xFFFFFFFF
                } else {
                    central_start as u32
                },
            };

            footer.write(writer)?;
        }

        Ok(())
    }
}

impl<W: Write + io::Seek> Drop for ZipWriter<W> {
    fn drop(&mut self) {
        if !self.inner.is_closed() {
            if let Err(e) = self.finalize() {
                let _ = write!(&mut io::stderr(), "ZipWriter drop failed: {:?}", e);
            }
        }
    }
}

impl<W: Write + io::Seek> GenericZipWriter<W> {
    fn switch_to(&mut self, compression: CompressionMethod) -> ZipResult<()> {
        match self.current_compression() {
            Some(method) if method == compression => return Ok(()),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "ZipWriter was already closed",
                )
                .into())
            }
            _ => {}
        }

        let bare = match mem::replace(self, GenericZipWriter::Closed) {
            GenericZipWriter::Storer(w) => w,
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(w) => w.finish()?,
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(w) => w.finish()?,
            GenericZipWriter::Closed => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "ZipWriter was already closed",
                )
                .into())
            }
        };

        *self = {
            #[allow(deprecated)]
            match compression {
                CompressionMethod::Stored => GenericZipWriter::Storer(bare),
                #[cfg(any(
                    feature = "deflate",
                    feature = "deflate-miniz",
                    feature = "deflate-zlib"
                ))]
                CompressionMethod::Deflated => GenericZipWriter::Deflater(DeflateEncoder::new(
                    bare,
                    flate2::Compression::default(),
                )),
                #[cfg(feature = "bzip2")]
                CompressionMethod::Bzip2 => {
                    GenericZipWriter::Bzip2(BzEncoder::new(bare, bzip2::Compression::default()))
                }
                CompressionMethod::Unsupported(..) => {
                    return Err(ZipError::UnsupportedArchive("Unsupported compression"))
                }
            }
        };

        Ok(())
    }

    fn ref_mut(&mut self) -> Option<&mut dyn Write> {
        match *self {
            GenericZipWriter::Storer(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(ref mut w) => Some(w as &mut dyn Write),
            GenericZipWriter::Closed => None,
        }
    }

    fn is_closed(&self) -> bool {
        match *self {
            GenericZipWriter::Closed => true,
            _ => false,
        }
    }

    fn get_plain(&mut self) -> &mut W {
        match *self {
            GenericZipWriter::Storer(ref mut w) => w,
            _ => panic!("Should have switched to stored beforehand"),
        }
    }

    fn current_compression(&self) -> Option<CompressionMethod> {
        match *self {
            GenericZipWriter::Storer(..) => Some(CompressionMethod::Stored),
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(..) => Some(CompressionMethod::Deflated),
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(..) => Some(CompressionMethod::Bzip2),
            GenericZipWriter::Closed => None,
        }
    }

    fn unwrap(self) -> W {
        match self {
            GenericZipWriter::Storer(w) => w,
            _ => panic!("Should have switched to stored beforehand"),
        }
    }
}

fn write_local_file_header<T: Write>(writer: &mut T, file: &ZipFileData) -> ZipResult<()> {
    // local file header signature
    writer.write_u32::<LittleEndian>(spec::LOCAL_FILE_HEADER_SIGNATURE)?;
    // version needed to extract
    writer.write_u16::<LittleEndian>(file.version_needed())?;
    // general purpose bit flag
    let flag = if !file.file_name.is_ascii() {
        1u16 << 11
    } else {
        0
    };
    writer.write_u16::<LittleEndian>(flag)?;
    // Compression method
    #[allow(deprecated)]
    writer.write_u16::<LittleEndian>(file.compression_method.to_u16())?;
    // last mod file time and last mod file date
    writer.write_u16::<LittleEndian>(file.last_modified_time.timepart())?;
    writer.write_u16::<LittleEndian>(file.last_modified_time.datepart())?;
    // crc-32
    writer.write_u32::<LittleEndian>(file.crc32)?;
    // compressed size
    writer.write_u32::<LittleEndian>(if file.compressed_size > 0xFFFFFFFF {
        0xFFFFFFFF
    } else {
        file.compressed_size as u32
    })?;
    // uncompressed size
    writer.write_u32::<LittleEndian>(if file.uncompressed_size > 0xFFFFFFFF {
        0xFFFFFFFF
    } else {
        file.uncompressed_size as u32
    })?;
    // file name length
    writer.write_u16::<LittleEndian>(file.file_name.as_bytes().len() as u16)?;
    // extra field length
    let extra_field_length = if file.large_file { 20 } else { 0 } + file.extra_field.len() as u16;
    writer.write_u16::<LittleEndian>(extra_field_length)?;
    // file name
    writer.write_all(file.file_name.as_bytes())?;
    // zip64 extra field
    if file.large_file {
        write_local_zip64_extra_field(writer, &file)?;
    }

    Ok(())
}

fn update_local_file_header<T: Write + io::Seek>(
    writer: &mut T,
    file: &ZipFileData,
) -> ZipResult<()> {
    const CRC32_OFFSET: u64 = 14;
    writer.seek(io::SeekFrom::Start(file.header_start + CRC32_OFFSET))?;
    writer.write_u32::<LittleEndian>(file.crc32)?;
    writer.write_u32::<LittleEndian>(if file.compressed_size > 0xFFFFFFFF {
        if file.large_file {
            0xFFFFFFFF
        } else {
            // compressed size can be slightly larger than uncompressed size
            return Err(ZipError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Large file option has not been set",
            )));
        }
    } else {
        file.compressed_size as u32
    })?;
    writer.write_u32::<LittleEndian>(if file.uncompressed_size > 0xFFFFFFFF {
        // uncompressed size is checked on write to catch it as soon as possible
        0xFFFFFFFF
    } else {
        file.uncompressed_size as u32
    })?;
    if file.large_file {
        update_local_zip64_extra_field(writer, file)?;
    }
    Ok(())
}

fn write_central_directory_header<T: Write>(writer: &mut T, file: &ZipFileData) -> ZipResult<()> {
    // buffer zip64 extra field to determine its variable length
    let mut zip64_extra_field = [0; 28];
    let zip64_extra_field_length =
        write_central_zip64_extra_field(&mut zip64_extra_field.as_mut(), file)?;

    // central file header signature
    writer.write_u32::<LittleEndian>(spec::CENTRAL_DIRECTORY_HEADER_SIGNATURE)?;
    // version made by
    let version_made_by = (file.system as u16) << 8 | (file.version_made_by as u16);
    writer.write_u16::<LittleEndian>(version_made_by)?;
    // version needed to extract
    writer.write_u16::<LittleEndian>(file.version_needed())?;
    // general puprose bit flag
    let flag = if !file.file_name.is_ascii() {
        1u16 << 11
    } else {
        0
    };
    writer.write_u16::<LittleEndian>(flag)?;
    // compression method
    #[allow(deprecated)]
    writer.write_u16::<LittleEndian>(file.compression_method.to_u16())?;
    // last mod file time + date
    writer.write_u16::<LittleEndian>(file.last_modified_time.timepart())?;
    writer.write_u16::<LittleEndian>(file.last_modified_time.datepart())?;
    // crc-32
    writer.write_u32::<LittleEndian>(file.crc32)?;
    // compressed size
    writer.write_u32::<LittleEndian>(if file.compressed_size > 0xFFFFFFFF {
        0xFFFFFFFF
    } else {
        file.compressed_size as u32
    })?;
    // uncompressed size
    writer.write_u32::<LittleEndian>(if file.uncompressed_size > 0xFFFFFFFF {
        0xFFFFFFFF
    } else {
        file.uncompressed_size as u32
    })?;
    // file name length
    writer.write_u16::<LittleEndian>(file.file_name.as_bytes().len() as u16)?;
    // extra field length
    writer.write_u16::<LittleEndian>(zip64_extra_field_length + file.extra_field.len() as u16)?;
    // file comment length
    writer.write_u16::<LittleEndian>(0)?;
    // disk number start
    writer.write_u16::<LittleEndian>(0)?;
    // internal file attribytes
    writer.write_u16::<LittleEndian>(0)?;
    // external file attributes
    writer.write_u32::<LittleEndian>(file.external_attributes)?;
    // relative offset of local header
    writer.write_u32::<LittleEndian>(if file.header_start > 0xFFFFFFFF {
        0xFFFFFFFF
    } else {
        file.header_start as u32
    })?;
    // file name
    writer.write_all(file.file_name.as_bytes())?;
    // zip64 extra field
    writer.write_all(&zip64_extra_field[..zip64_extra_field_length as usize])?;
    // extra field
    writer.write_all(&file.extra_field)?;
    // file comment
    // <none>

    Ok(())
}

fn validate_extra_data(file: &ZipFileData) -> ZipResult<()> {
    let mut data = file.extra_field.as_slice();

    if data.len() > 0xFFFF {
        return Err(ZipError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "Extra data exceeds extra field",
        )));
    }

    while data.len() > 0 {
        let left = data.len();
        if left < 4 {
            return Err(ZipError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Incomplete extra data header",
            )));
        }
        let kind = data.read_u16::<LittleEndian>()?;
        let size = data.read_u16::<LittleEndian>()? as usize;
        let left = left - 4;

        if kind == 0x0001 {
            return Err(ZipError::Io(io::Error::new(
                io::ErrorKind::Other,
                "No custom ZIP64 extra data allowed",
            )));
        }

        #[cfg(not(feature = "unreserved"))]
        {
            if kind <= 31 || EXTRA_FIELD_MAPPING.iter().any(|&mapped| mapped == kind) {
                return Err(ZipError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Extra data header ID {:#06} requires crate feature \"unreserved\"",
                        kind,
                    ),
                )));
            }
        }

        if size > left {
            return Err(ZipError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Extra data size exceeds extra field",
            )));
        }

        data = &data[size..];
    }

    Ok(())
}

fn write_local_zip64_extra_field<T: Write>(writer: &mut T, file: &ZipFileData) -> ZipResult<()> {
    // This entry in the Local header MUST include BOTH original
    // and compressed file size fields.
    writer.write_u16::<LittleEndian>(0x0001)?;
    writer.write_u16::<LittleEndian>(16)?;
    writer.write_u64::<LittleEndian>(file.uncompressed_size)?;
    writer.write_u64::<LittleEndian>(file.compressed_size)?;
    // Excluded fields:
    // u32: disk start number
    Ok(())
}

fn update_local_zip64_extra_field<T: Write + io::Seek>(
    writer: &mut T,
    file: &ZipFileData,
) -> ZipResult<()> {
    let zip64_extra_field = file.header_start + 30 + file.file_name.as_bytes().len() as u64;
    writer.seek(io::SeekFrom::Start(zip64_extra_field + 4))?;
    writer.write_u64::<LittleEndian>(file.uncompressed_size)?;
    writer.write_u64::<LittleEndian>(file.compressed_size)?;
    // Excluded fields:
    // u32: disk start number
    Ok(())
}

fn write_central_zip64_extra_field<T: Write>(writer: &mut T, file: &ZipFileData) -> ZipResult<u16> {
    // The order of the fields in the zip64 extended
    // information record is fixed, but the fields MUST
    // only appear if the corresponding Local or Central
    // directory record field is set to 0xFFFF or 0xFFFFFFFF.
    let mut size = 0;
    let uncompressed_size = file.uncompressed_size > 0xFFFFFFFF;
    let compressed_size = file.compressed_size > 0xFFFFFFFF;
    let header_start = file.header_start > 0xFFFFFFFF;
    if uncompressed_size {
        size += 8;
    }
    if compressed_size {
        size += 8;
    }
    if header_start {
        size += 8;
    }
    if size > 0 {
        writer.write_u16::<LittleEndian>(0x0001)?;
        writer.write_u16::<LittleEndian>(size)?;
        size += 4;

        if uncompressed_size {
            writer.write_u64::<LittleEndian>(file.uncompressed_size)?;
        }
        if compressed_size {
            writer.write_u64::<LittleEndian>(file.compressed_size)?;
        }
        if header_start {
            writer.write_u64::<LittleEndian>(file.header_start)?;
        }
        // Excluded fields:
        // u32: disk start number
    }
    Ok(size)
}

fn path_to_string(path: &std::path::Path) -> String {
    let mut path_str = String::new();
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if !path_str.is_empty() {
                path_str.push('/');
            }
            path_str.push_str(&*os_str.to_string_lossy());
        }
    }
    path_str
}

#[cfg(test)]
mod test {
    use super::{FileOptions, ZipWriter};
    use crate::compression::CompressionMethod;
    use crate::types::DateTime;
    use std::io;
    use std::io::Write;

    #[test]
    fn write_empty_zip() {
        let mut writer = ZipWriter::new(io::Cursor::new(Vec::new()));
        writer.set_comment("ZIP");
        let result = writer.finish().unwrap();
        assert_eq!(result.get_ref().len(), 25);
        assert_eq!(
            *result.get_ref(),
            [80, 75, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 90, 73, 80]
        );
    }

    #[test]
    fn write_zip_dir() {
        let mut writer = ZipWriter::new(io::Cursor::new(Vec::new()));
        writer
            .add_directory(
                "test",
                FileOptions::default().last_modified_time(
                    DateTime::from_date_and_time(2018, 8, 15, 20, 45, 6).unwrap(),
                ),
            )
            .unwrap();
        assert!(writer
            .write(b"writing to a directory is not allowed, and will not write any data")
            .is_err());
        let result = writer.finish().unwrap();
        assert_eq!(result.get_ref().len(), 108);
        assert_eq!(
            *result.get_ref(),
            &[
                80u8, 75, 3, 4, 20, 0, 0, 0, 0, 0, 163, 165, 15, 77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 47, 80, 75, 1, 2, 46, 3, 20, 0, 0, 0, 0, 0,
                163, 165, 15, 77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 237, 65, 0, 0, 0, 0, 116, 101, 115, 116, 47, 80, 75, 5, 6, 0, 0, 0, 0, 1, 0,
                1, 0, 51, 0, 0, 0, 35, 0, 0, 0, 0, 0,
            ] as &[u8]
        );
    }

    #[test]
    fn write_mimetype_zip() {
        let mut writer = ZipWriter::new(io::Cursor::new(Vec::new()));
        let options = FileOptions {
            compression_method: CompressionMethod::Stored,
            last_modified_time: DateTime::default(),
            permissions: Some(33188),
            large_file: false,
        };
        writer.start_file("mimetype", options).unwrap();
        writer
            .write(b"application/vnd.oasis.opendocument.text")
            .unwrap();
        let result = writer.finish().unwrap();

        assert_eq!(result.get_ref().len(), 153);
        let mut v = Vec::new();
        v.extend_from_slice(include_bytes!("../tests/data/mimetype.zip"));
        assert_eq!(result.get_ref(), &v);
    }

    #[test]
    fn path_to_string() {
        let mut path = std::path::PathBuf::new();
        #[cfg(windows)]
        path.push(r"C:\");
        #[cfg(unix)]
        path.push("/");
        path.push("windows");
        path.push("..");
        path.push(".");
        path.push("system32");
        let path_str = super::path_to_string(&path);
        assert_eq!(path_str, "windows/system32");
    }
}

#[cfg(not(feature = "unreserved"))]
const EXTRA_FIELD_MAPPING: [u16; 49] = [
    0x0001, 0x0007, 0x0008, 0x0009, 0x000a, 0x000c, 0x000d, 0x000e, 0x000f, 0x0014, 0x0015, 0x0016,
    0x0017, 0x0018, 0x0019, 0x0020, 0x0021, 0x0022, 0x0023, 0x0065, 0x0066, 0x4690, 0x07c8, 0x2605,
    0x2705, 0x2805, 0x334d, 0x4341, 0x4453, 0x4704, 0x470f, 0x4b46, 0x4c41, 0x4d49, 0x4f4c, 0x5356,
    0x5455, 0x554e, 0x5855, 0x6375, 0x6542, 0x7075, 0x756e, 0x7855, 0xa11e, 0xa220, 0xfd4a, 0x9901,
    0x9902,
];
