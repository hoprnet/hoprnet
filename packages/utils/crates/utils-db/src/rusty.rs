use async_trait::async_trait;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::errors::DbError;
use crate::traits::{AsyncKVStorage, BatchOperation, StorageValueIterator};
use futures_lite::stream::iter;
use hex_literal::hex;
use rusty_leveldb::{DBIterator, Env, LdbIterator, StatusCode, WriteBatch};
use utils_log::debug;

struct RustyLevelDbIterator {
    iter: DBIterator,
    first_key: Box<[u8]>,
    last_key: Box<[u8]>,
}

impl RustyLevelDbIterator {
    pub fn new(iter: DBIterator, prefix: &[u8], suffix_len: usize) -> Self {
        let mut first_key: Vec<u8> = prefix.into();
        first_key.extend((0..suffix_len).map(|_| 0u8));

        let mut last_key: Vec<u8> = prefix.into();
        last_key.extend((0..suffix_len).map(|_| 0xffu8));

        // This implementation does not use the `seek` method, because it is not working properly
        Self {
            iter,
            first_key: first_key.into_boxed_slice(),
            last_key: last_key.into_boxed_slice(),
        }
    }
}

impl Iterator for RustyLevelDbIterator {
    type Item = crate::errors::Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((key, value)) = self.iter.next() {
            let upper_bound = key.as_slice().cmp(&self.last_key);
            let lower_bound = key.as_slice().cmp(&self.first_key);
            if upper_bound != Ordering::Greater && lower_bound != Ordering::Less {
                return Some(Ok(value.into_boxed_slice()));
            } else if upper_bound == Ordering::Greater {
                return None;
            }
        }
        None
    }
}

/// Adapter for Rusty Level DB database.
#[derive(Clone)]
pub struct RustyLevelDbShim {
    db: Arc<Mutex<rusty_leveldb::DB>>,
}

impl RustyLevelDbShim {
    #[cfg(not(feature = "wasm"))]
    pub fn new_in_memory() -> Self {
        Self {
            db: Arc::new(Mutex::new(rusty_leveldb::DB::open("hoprd_db", rusty_leveldb::in_memory())
                .expect("failed to create DB")))
        }
    }

    #[cfg(feature = "wasm")]
    pub fn new_in_memory() -> Self {
        Self {
            db: Arc::new(Mutex::new(rusty_leveldb::DB::open("hoprd_db", wasm::WasmMemEnv::create_options())
                .expect("failed to create DB")))
        }
    }

    #[cfg(feature = "wasm")]
    pub fn new(path: &str) -> Self {
        Self {
            db: Arc::new(Mutex::new(rusty_leveldb::DB::open(path, wasm::NodeJsEnv::create_options())
                .expect("failed to create DB")))
        }
    }
}

#[async_trait(?Send)]
impl AsyncKVStorage for RustyLevelDbShim {
    type Key = Box<[u8]>;
    type Value = Box<[u8]>;

    async fn get(&self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        let mut db = self.db.lock().unwrap();

        let snapshot = db.get_snapshot();
        match db.get_at(&snapshot, &key) {
            Ok(Some(val)) => Ok(if val.len() > 0 {
                Some(val.into_boxed_slice())
            } else {
                None
            }),
            Ok(None) => Ok(None),
            Err(e) => Err(if e.code == StatusCode::NotFound {
                DbError::NotFound
            } else {
                DbError::GenericError(e.to_string())
            }),
        }
    }

    async fn set(&mut self, key: Self::Key, value: Self::Value) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .lock()
            .unwrap()
            .put(&key, &value)
            .map(|_| None)
            .map_err(|e| DbError::GenericError(e.err))
    }

    async fn contains(&self, key: Self::Key) -> crate::errors::Result<bool> {
        Ok(self.db.lock().unwrap().get(&key).is_some())
    }

    async fn remove(&mut self, key: Self::Key) -> crate::errors::Result<Option<Self::Value>> {
        self.db
            .lock()
            .unwrap()
            .delete(&key)
            .map(|_| None)
            .map_err(|e| DbError::GenericError(e.err))
    }

    async fn dump(&self, _destination: String) -> crate::errors::Result<()> {
        Ok(())
    }

    fn iterate(
        &self,
        prefix: Self::Key,
        suffix_size: u32,
    ) -> crate::errors::Result<StorageValueIterator<Self::Value>> {
        let i = self
            .db
            .lock()
            .unwrap()
            .new_iter()
            .map_err(|e| DbError::GenericError(e.err))?;
        Ok(Box::new(iter(RustyLevelDbIterator::new(
            i,
            &prefix,
            suffix_size as usize,
        ))))
    }

    async fn batch(
        &mut self,
        operations: Vec<BatchOperation<Self::Key, Self::Value>>,
        wait_for_write: bool,
    ) -> crate::errors::Result<()> {
        let mut wb = WriteBatch::new();
        for op in operations {
            match op {
                BatchOperation::del(x) => wb.delete(&x.key),
                BatchOperation::put(x) => wb.put(&x.key, &x.value),
            }
        }

        self.db
            .lock()
            .unwrap()
            .write(wb, wait_for_write)
            .map_err(|e| DbError::GenericError(e.err))
    }
}

#[cfg(any(feature = "wasm", test))]
fn test_env(env: Box<dyn Env>, base: &Path, ts: u64) {
    debug!("test #1");
    let test_dir = base.join(format!("test_dir_{0}", ts));
    env.mkdir(&test_dir).expect("could not create dir");

    debug!("test #2");
    let test_file = test_dir.join("test_file");
    assert!(!env.exists(&test_file).expect("could not check file existence 1"), "file should not exist before creation");

    debug!("test #3");
    let data = hex!("deadbeefcafebabe");

    {
        debug!("test #4");
        let mut f = env.open_writable_file(&test_file).expect("could not open file 1");
        let len = f.write(&data).expect("could not write to a file");
        assert_eq!(data.len(), len, "writting invalid number of bytes");
    }

    {
        debug!("test #5");
        assert!(env.exists(&test_file).expect("could not check file existence 3"), "file should exist");
        let mut f = env.open_sequential_file(&test_file).expect("could not open file 2");
        let mut buf = vec![0u8; data.len()];
        let len = f.read(&mut buf).expect("could not read from file");
        assert_eq!(data.len(), len, "could not read all bytes from the file 2");
        assert_eq!(data, buf.as_slice(), "read incorrect data");
    }

    {
        debug!("test #6");
        let mut f = env.open_appendable_file(&test_file).expect("could not open file 3");
        let len = f.write(&data).expect("appendable write failed");
        assert_eq!(data.len(), len, "could not write all bytes to the file");
    }

    {
        debug!("test #7");
        let len = env.size_of(&test_file).expect("could not get file size");
        assert_eq!(data.len() * 2, len, "file should have twice the length after appending");
    }

    {
        debug!("test #8");
        let f = env.open_random_access_file(&test_file).expect("could not open file 4");
        let mut buf = [0; 8];
        let len = f.read_at(4, &mut buf).expect("could not read file at 1");
        assert_eq!(len, buf.len(), "could not read all bytes 3");
        assert_eq!(hex!("cafebabedeadbeef"), buf);

        let mut buf = [0; 4];
        let len = f.read_at(4, &mut buf).expect("could not read file at 2");
        assert_eq!(len, buf.len(), "could not read all bytes 4");
        assert_eq!(hex!("cafebabe"), buf, "mismatch random access read bytes 1");

        let mut buf = [0; 4];
        let len = f.read_at(2, &mut buf).expect("could not read file at 4");
        assert_eq!(len, buf.len(), "could not read all bytes 6");
        assert_eq!(hex!("beefcafe"), buf, "mismatch random access read bytes 2");
    }

    {
        debug!("test #9");
        let children = env.children(&test_dir).expect("cannot retrieve children of test dir");
        assert_eq!(children.len(), 1, "contains more children");
        assert!(children.contains(&PathBuf::from("test_file".to_string())), "children do not contain test file");
    }

    let new_file = test_dir.join("new_file");
    {
        debug!("test #10");
        env.rename(&test_file, &new_file).expect("rename must not fail");
        assert!(!env.exists(&test_file).expect("failed to check existence after rename"), "old file must not exist after rename");
        assert!(env.exists(&new_file).expect("failed to check existence after rename"), "new file must exist after rename");
    }

    {
        debug!("test #11");
        env.delete(&new_file).expect("could not delete file");
        assert!(!env.exists(&new_file).expect("failed to check existence after deletion"), "file must not exist after deletion");
    }

    {
        debug!("test #12");
        let _ = env.rmdir(&test_dir); // cannot be tested with MemEnv
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use rusty_leveldb::MemEnv;
    use crate::rusty::test_env;

    #[async_std::test]
    async fn rusty_leveldb_sanity_test() {
        use crate::traits::{AsyncKVStorage, BatchOperation};
        use futures_lite::StreamExt;

        let key_1 = "1";
        let value_1 = "abc";
        let key_2 = "2";
        let value_2 = "def";
        let key_3 = "3";
        let value_3 = "ghi";
        let key_4 = "1";
        let prefix = "xy";
        let prefixed_key_1 = "xya";
        let prefixed_key_2 = "xyb";
        let prefixed_key_3 = "xyc";

        let mut kv_storage = crate::rusty::RustyLevelDbShim::new_in_memory();

        assert!(
            !kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #1 failed: empty DB should not contain any data"
        );

        let _ = kv_storage.set(key_1.as_bytes().into(), value_1.as_bytes().into()).await;

        assert!(
            kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #2 failed: DB should contain the key"
        );

        let value = kv_storage
            .get(key_1.as_bytes().into())
            .await
            .unwrap()
            .expect("Stored empty value");
        let value_converted = std::str::from_utf8(value.as_ref()).unwrap();

        assert_eq!(
            value_converted, value_1,
            "Test #3 failed: DB value after get should be equal to the one before the get"
        );

        let _ = kv_storage.remove(key_1.as_bytes().into()).await;
        assert!(
            !kv_storage.contains(key_1.as_bytes().into()).await.unwrap(),
            "Test #4 failed: removal of key from the DB failed"
        );

        let batch_data = vec![
            BatchOperation::put(crate::traits::Put {
                key: key_3.as_bytes().into(),
                value: value_3.as_bytes().into(),
            }),
            BatchOperation::put(crate::traits::Put {
                key: key_2.as_bytes().into(),
                value: value_2.as_bytes().into(),
            }),
            BatchOperation::del(crate::traits::Del {
                key: key_2.as_bytes().into(),
            }),
        ];
        assert!(
            kv_storage.batch(batch_data, true).await.is_ok(),
            "Test #5.0 failed: batch operation failed"
        );

        // ===================================

        async_std::task::sleep(std::time::Duration::from_millis(10)).await;

        assert!(
            kv_storage.contains(key_3.as_bytes().into()).await.unwrap(),
            "Test #5.1 failed: the key should be present in the DB"
        );

        kv_storage
            .set(key_4.as_bytes().into(), Box::new([]))
            .await
            .expect("Could not write empty value");

        assert!(kv_storage.contains(key_4.as_bytes().into()).await.unwrap());

        assert_eq!(
            kv_storage.get(key_4.as_bytes().into()).await,
            Ok(None),
            "Test #6 failed: Could not read empty value from DB"
        );

        // ===================================

        let _ = kv_storage
            .set(prefixed_key_1.as_bytes().into(), value_1.as_bytes().into())
            .await;
        let _ = kv_storage
            .set(prefixed_key_2.as_bytes().into(), value_2.as_bytes().into())
            .await;
        let _ = kv_storage
            .set(prefixed_key_3.as_bytes().into(), value_3.as_bytes().into())
            .await;

        let expected = vec![value_1.as_bytes().into(), value_3.as_bytes().into()];

        let mut received = Vec::new();
        let mut data_stream = Box::into_pin(
            kv_storage
                .iterate(prefix.as_bytes().into(), (prefixed_key_1.len() - prefix.len()) as u32)
                .unwrap(),
        );

        while let Some(value) = data_stream.next().await {
            let v = value.unwrap();

            if v.as_ref() != value_2.as_bytes() {
                received.push(v);
            }
        }
        assert_eq!(received, expected, "Test #7 failed: db content mismatch");
    }


    #[test]
    fn wasm_test_sanity() {
        // This just tests the sanity of the "test_env" against a known working implementation (MemEnv)
        // so it can be further used in the WASM test
        test_env(Box::new(MemEnv::new()), Path::new("/"), utils_misc::time::native::current_timestamp())
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::{io, thread, time};
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use js_sys::{JsString, Uint8Array};
    use rusty_leveldb::{Env, FileLock, Logger, MemEnv, RandomAccess, Status, StatusCode};
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::rusty::test_env;

    pub struct WasmMemEnv(MemEnv);

    impl WasmMemEnv {
        pub fn create_options() -> rusty_leveldb::Options {
            let mut opt = rusty_leveldb::Options::default();
            opt.env = Rc::new(Box::new(WasmMemEnv(MemEnv::new())));
            opt
        }
    }

    impl Env for WasmMemEnv {
        fn open_sequential_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Read>> {
            self.0.open_sequential_file(p)
        }

        fn open_random_access_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn rusty_leveldb::RandomAccess>> {
            self.0.open_random_access_file(p)
        }

        fn open_writable_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Write>> {
            self.0.open_writable_file(p)
        }

        fn open_appendable_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Write>> {
            self.0.open_appendable_file(p)
        }

        fn exists(&self, p: &Path) -> rusty_leveldb::Result<bool> {
            self.0.exists(p)
        }

        fn children(&self, p: &Path) -> rusty_leveldb::Result<Vec<PathBuf>> {
            self.0.children(p)
        }

        fn size_of(&self, p: &Path) -> rusty_leveldb::Result<usize> {
            self.0.size_of(p)
        }

        fn delete(&self, p: &Path) -> rusty_leveldb::Result<()> {
            self.0.delete(p)
        }

        fn mkdir(&self, p: &Path) -> rusty_leveldb::Result<()> {
            self.0.mkdir(p)
        }

        fn rmdir(&self, p: &Path) -> rusty_leveldb::Result<()> {
            self.0.rmdir(p)
        }

        fn rename(&self, old: &Path, new: &Path) -> rusty_leveldb::Result<()> {
            self.0.rename(old, new)
        }

        fn lock(&self, p: &Path) -> rusty_leveldb::Result<rusty_leveldb::FileLock> {
            self.0.lock(p)
        }

        fn unlock(&self, l: rusty_leveldb::FileLock) -> rusty_leveldb::Result<()> {
            self.0.unlock(l)
        }

        fn new_logger(&self, p: &Path) -> rusty_leveldb::Result<rusty_leveldb::Logger> {
            self.0.new_logger(p)
        }

        fn micros(&self) -> u64 {
            utils_misc::time::wasm::current_timestamp() * 1000
        }

        fn sleep_for(&self, micros: u32) {
            self.0.sleep_for(micros)
        }
    }

    #[wasm_bindgen(module = "fs")]
    extern "C" {
        #[derive(Debug, Clone)]
        pub type Stats;

        #[wasm_bindgen(method, getter)]
        fn dev(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn ino(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn mode(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn nlink(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn uid(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn gid(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn rdev(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn size(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn blksize(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn blocks(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn atimeMs(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn mtimeMs(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn ctimeMs(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn birthtimeMs(this: &Stats) -> u32;
        #[wasm_bindgen(method, getter)]
        fn atime(this: &Stats) -> js_sys::Date;
        #[wasm_bindgen(method, getter)]
        fn mtime(this: &Stats) -> js_sys::Date;
        #[wasm_bindgen(method, getter)]
        fn ctime(this: &Stats) -> js_sys::Date;
        #[wasm_bindgen(method, getter)]
        fn birthtime(this: &Stats) -> js_sys::Date;
    }

    #[wasm_bindgen(module = "fs")]
    extern "C" {
        #[wasm_bindgen(catch)]
        fn existsSync(path: &str) -> Result<bool, JsValue>;
        #[wasm_bindgen(catch)]
        fn openSync(path: &str, flags: Option<JsString>, mode: Option<JsString>) -> Result<i32, JsValue>;
        #[wasm_bindgen(catch)]
        fn readSync(fd: i32, buffer: &Uint8Array, offset: u32, length: u32, position: Option<u32>) -> Result<i32, JsValue>;
        #[wasm_bindgen(catch)]
        fn writeSync(fd: i32, buffer: &Uint8Array, offset: u32, length: Option<u32>, position: Option<u32>) -> Result<i32, JsValue>;
        #[wasm_bindgen(catch)]
        fn fsyncSync(fd: i32) -> Result<(), JsValue>;
        #[wasm_bindgen(catch)]
        fn fstatSync(fd: i32, options: &JsValue) -> Result<Stats, JsValue>;
        #[wasm_bindgen(catch)]
        fn closeSync(fd: i32) -> Result<(), JsValue>;
        #[wasm_bindgen(catch)]
        fn mkdirSync(path: &str, options: &JsValue) -> Result<JsString, JsValue>;
        #[wasm_bindgen(catch)]
        fn rmdirSync(path: &str, options: &JsValue) -> Result<(), JsValue>;
        #[wasm_bindgen(catch)]
        fn rmSync(path: &str, options: &JsValue) -> Result<(), JsValue>;
        #[wasm_bindgen(catch)]
        fn readdirSync(path: &str, options: &JsValue) -> Result<Vec<JsString>, JsValue>;
        #[wasm_bindgen(catch)]
        fn renameSync(old: &str, new: &str) -> Result<(), JsValue>;
    }

    struct FileHandle(i32);

    impl FileHandle {
        pub fn open(path: &str, flags: Option<String>) -> rusty_leveldb::Result<Self> {
            let fd = openSync(path, flags.map(JsString::from), None)
                .map_err(|v| Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in open".into())))?;
            if fd >= 0 {
                Ok(Self(fd))
            } else {
                Err(Status::new(StatusCode::IOError, &format!("could not open file {path}")))
            }
        }

        fn read_from(&self, offset: Option<u32>, dst: &mut [u8]) -> rusty_leveldb::Result<usize> {
            let mut ubuf = Uint8Array::new_with_length(dst.len() as u32);
            let read = readSync(self.0, &mut ubuf, 0, dst.len() as u32, offset)
                .map_err(|v| Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in read".into())))?;

            if read > 0 {
                ubuf.copy_to(dst);
                Ok(read as usize)
            } else if read == 0 {
                Ok(0)
            } else {
                Err(Status::new(StatusCode::IOError, "read error"))
            }
        }
    }

    impl Read for FileHandle {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read_from(None, buf)
                .map_err(|_| io::ErrorKind::Other.into())
        }
    }

    impl Write for FileHandle {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let arr = Uint8Array::new_with_length(buf.len() as u32);
            arr.copy_from(buf);

            let written = writeSync(self.0, &arr, 0, Some(buf.len() as u32), None)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

            if written >= 0 {
                Ok(written as usize)
            } else {
                Err(io::ErrorKind::Other.into())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            fsyncSync(self.0).map_err(|_| io::Error::from(io::ErrorKind::Other))
        }
    }

    impl RandomAccess for FileHandle {
        fn read_at(&self, off: usize, dst: &mut [u8]) -> rusty_leveldb::Result<usize> {
            self.read_from(Some(off as u32), dst)
        }
    }

    impl Drop for FileHandle {
        fn drop(&mut self) {
            let _ = fsyncSync(self.0);
            let _ = closeSync(self.0);
        }
    }

    pub struct NodeJsEnv {
        locks: Arc<Mutex<HashMap<String, FileHandle>>>,
    }

    impl NodeJsEnv {
        pub fn create_options() -> rusty_leveldb::Options {
            let mut opt = rusty_leveldb::Options::default();
            opt.env = Rc::new(Box::new(Self::new()));
            opt
        }

        pub fn new() -> Self {
            Self {
                locks: Arc::new(Mutex::new(HashMap::new()))
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Opts {
        recursive: bool,
    }

    impl Env for NodeJsEnv {
        fn open_sequential_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Read>> {
            Ok(FileHandle::open(p.to_str().expect("invalid path"), Some("r".into()))
                .map(Box::new)
                .map_err(rusty_leveldb::Status::from)?)
        }

        fn open_random_access_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn RandomAccess>> {
            Ok(FileHandle::open(p.to_str().expect("invalid path"), Some("r".into()))
                .map(Box::new)
                .map_err(rusty_leveldb::Status::from)?)
        }

        fn open_writable_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Write>> {
            Ok(FileHandle::open(p.to_str().expect("invalid path"), Some("w".into()))
                .map(Box::new)
                .map_err(rusty_leveldb::Status::from)?)
        }

        fn open_appendable_file(&self, p: &Path) -> rusty_leveldb::Result<Box<dyn Write>> {
            Ok(FileHandle::open(p.to_str().expect("invalid path"), Some("a".into()))
                .map(Box::new)
                .map_err(rusty_leveldb::Status::from)?)
        }

        fn exists(&self, p: &Path) -> rusty_leveldb::Result<bool> {
            existsSync(p.to_str().expect("invalid path"))
                .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in exists".into())))
        }

        fn children(&self, p: &Path) -> rusty_leveldb::Result<Vec<PathBuf>> {
            Ok(readdirSync(p.to_str().expect("invalid path"), &JsValue::undefined())
                .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in readdir".into())))?
                .into_iter()
                .map(|s| PathBuf::from(s.as_string().expect("invalid path buf")))
                .collect())
        }

        fn size_of(&self, p: &Path) -> rusty_leveldb::Result<usize> {
            let fh = FileHandle::open(p.to_str().expect("invalid file path"), Some("r".into()))?;
            fstatSync(fh.0, &JsValue::undefined())
                .map(|s| s.size() as usize)
                .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in delete".into())))
        }

        fn delete(&self, p: &Path) -> rusty_leveldb::Result<()> {
            rmSync(p.to_str().expect("invalid path"), &JsValue::undefined())
                .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in delete".into())))
        }

        fn mkdir(&self, p: &Path) -> rusty_leveldb::Result<()> {
            let opts = serde_wasm_bindgen::to_value(&Opts { recursive: true })
                .map_err(|_| Status::new(StatusCode::IOError, "failed to convert opts"))?;

            if let Err(e) = mkdirSync(p.to_str().expect("invalid path"), &opts).map(|_| ()) {
                let err_str = e.as_string().unwrap_or("unknown error in mkdir".into());
                if err_str.contains("EEXIST") { // don't fail if path already exists
                    Ok(())
                } else {
                    Err(Status::new(StatusCode::IOError, &err_str))
                }
            } else {
                Ok(())
            }
        }

        fn rmdir(&self, p: &Path) -> rusty_leveldb::Result<()> {

            let opts = serde_wasm_bindgen::to_value(&Opts { recursive: true })
                .map_err(|_| Status::new(StatusCode::IOError, "failed to convert opts"))?;

            rmdirSync(p.to_str().expect("invalid path"), &opts)
                .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in rmdir".into())))
        }

        fn rename(&self, old: &Path, new: &Path) -> rusty_leveldb::Result<()> {
            renameSync(
                old.to_str().expect("invalid old path"),
                new.to_str().expect("invalid new path")
            )
            .map_err(|v|Status::new(StatusCode::IOError, &v.as_string().unwrap_or("unknown error in rename".into())))
        }

        fn lock(&self, p: &Path) -> rusty_leveldb::Result<FileLock> {
            let mut locks = self.locks.lock().unwrap();

            if locks.contains_key(&p.to_str().unwrap().to_string()) {
                Err(Status::new(StatusCode::AlreadyExists, "Lock is held"))
            } else {
                let lock_file = FileHandle(0);

                // TODO: implement proper file locking!

                locks.insert(p.to_str().unwrap().to_string(), lock_file);
                let lock = FileLock {
                    id: p.to_str().unwrap().to_string(),
                };
                Ok(lock)
            }
        }

        fn unlock(&self, l: FileLock) -> rusty_leveldb::Result<()> {
            let mut locks = self.locks.lock().unwrap();
            if !locks.contains_key(&l.id) {
                return Err(Status::new(
                    StatusCode::LockError,
                    "lock on database is already held by different process",
                ));
            } else {
                let _ = locks.remove(&l.id).unwrap();
                // TODO: implement proper file locking!
                Ok(())
            }
        }

        fn new_logger(&self, p: &Path) -> rusty_leveldb::Result<Logger> {
            self.open_appendable_file(p)
                .map(|dst| Logger::new(Box::new(dst)))
        }

        fn micros(&self) -> u64 {
            utils_misc::time::wasm::current_timestamp() * 1000
        }

        fn sleep_for(&self, micros: u32) {
            thread::sleep(time::Duration::new(0, micros * 1000));
        }
    }
    
    #[wasm_bindgen]
    pub fn test_nodejs_env(base_dir: &str) {
        test_env(Box::new(NodeJsEnv::new()), Path::new(base_dir), utils_misc::time::wasm::current_timestamp());
    }
}
