use bencher::{benchmark_group, benchmark_main};

use std::fs;
use std::io::{self, prelude::*, Cursor};

use bencher::Bencher;
use getrandom::getrandom;
use tempdir::TempDir;
use zip::write::SimpleFileOptions;
use zip::{result::ZipResult, CompressionMethod, ZipArchive, ZipWriter};

const FILE_COUNT: usize = 15_000;
const FILE_SIZE: usize = 1024;

fn generate_random_archive(count_files: usize, file_size: usize) -> ZipResult<Vec<u8>> {
    let data = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(data));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    let mut bytes = vec![0u8; file_size];

    for i in 0..count_files {
        let name = format!("file_deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef_{i}.dat");
        writer.start_file(name, options)?;
        getrandom(&mut bytes).map_err(io::Error::from)?;
        writer.write_all(&bytes)?;
    }

    Ok(writer.finish()?.into_inner())
}

fn read_metadata(bench: &mut Bencher) {
    let bytes = generate_random_archive(FILE_COUNT, FILE_SIZE).unwrap();

    bench.iter(|| {
        let archive = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        archive.len()
    });
    bench.bytes = bytes.len() as u64;
}

const COMMENT_SIZE: usize = 50_000;

fn generate_zip32_archive_with_random_comment(comment_length: usize) -> ZipResult<Vec<u8>> {
    let data = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(data));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    let mut bytes = vec![0u8; comment_length];
    getrandom(&mut bytes).unwrap();
    writer.set_raw_comment(bytes.into_boxed_slice());

    writer.start_file("asdf.txt", options)?;
    writer.write_all(b"asdf")?;

    Ok(writer.finish()?.into_inner())
}

fn parse_archive_with_comment(bench: &mut Bencher) {
    let bytes = generate_zip32_archive_with_random_comment(COMMENT_SIZE).unwrap();

    bench.bench_n(1, |_| {
        let archive = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        let _ = archive.comment().len();
    });
    bench.bytes = bytes.len() as u64;
}

const COMMENT_SIZE_64: usize = 500_000;

fn generate_zip64_archive_with_random_comment(comment_length: usize) -> ZipResult<Vec<u8>> {
    let data = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(data));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);

    let mut bytes = vec![0u8; comment_length];
    getrandom(&mut bytes).unwrap();
    writer.set_raw_comment(bytes.into_boxed_slice());

    writer.start_file("asdf.txt", options)?;
    writer.write_all(b"asdf")?;

    Ok(writer.finish()?.into_inner())
}

fn parse_zip64_archive_with_comment(bench: &mut Bencher) {
    let bytes = generate_zip64_archive_with_random_comment(COMMENT_SIZE_64).unwrap();

    bench.iter(|| {
        let archive = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        archive.comment().len()
    });
    bench.bytes = bytes.len() as u64;
}

fn parse_stream_archive(bench: &mut Bencher) {
    const STREAM_ZIP_ENTRIES: usize = 5;
    const STREAM_FILE_SIZE: usize = 5;

    let bytes = generate_random_archive(STREAM_ZIP_ENTRIES, STREAM_FILE_SIZE).unwrap();

    /* Write to a temporary file path to incur some filesystem overhead from repeated reads */
    let dir = TempDir::new("stream-bench").unwrap();
    let out = dir.path().join("bench-out.zip");
    fs::write(&out, &bytes).unwrap();

    bench.iter(|| {
        let mut f = fs::File::open(&out).unwrap();
        while zip::read::read_zipfile_from_stream(&mut f)
            .unwrap()
            .is_some()
        {}
    });
    bench.bytes = bytes.len() as u64;
}

benchmark_group!(
    benches,
    read_metadata,
    parse_archive_with_comment,
    parse_zip64_archive_with_comment,
    parse_stream_archive,
);
benchmark_main!(benches);
