use futures_core::future::BoxFuture;
use futures_intrusive::sync::MutexGuard;
use futures_util::future;
use libsqlite3_sys::{sqlite3, sqlite3_progress_handler};
use sqlx_core::common::StatementCache;
use sqlx_core::error::Error;
use sqlx_core::transaction::Transaction;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::os::raw::{c_int, c_void};
use std::panic::catch_unwind;
use std::ptr::NonNull;

use crate::connection::establish::EstablishParams;
use crate::connection::worker::ConnectionWorker;
use crate::options::OptimizeOnClose;
use crate::statement::VirtualStatement;
use crate::{Sqlite, SqliteConnectOptions};
use sqlx_core::executor::Executor;
use std::fmt::Write;

pub(crate) use sqlx_core::connection::*;

pub(crate) use handle::{ConnectionHandle, ConnectionHandleRaw};

pub(crate) mod collation;
pub(crate) mod describe;
pub(crate) mod establish;
pub(crate) mod execute;
mod executor;
mod explain;
mod handle;
mod intmap;

mod worker;

/// A connection to an open [Sqlite] database.
///
/// Because SQLite is an in-process database accessed by blocking API calls, SQLx uses a background
/// thread and communicates with it via channels to allow non-blocking access to the database.
///
/// Dropping this struct will signal the worker thread to quit and close the database, though
/// if an error occurs there is no way to pass it back to the user this way.
///
/// You can explicitly call [`.close()`][Self::close] to ensure the database is closed successfully
/// or get an error otherwise.
pub struct SqliteConnection {
    optimize_on_close: OptimizeOnClose,
    pub(crate) worker: ConnectionWorker,
    pub(crate) row_channel_size: usize,
}

pub struct LockedSqliteHandle<'a> {
    pub(crate) guard: MutexGuard<'a, ConnectionState>,
}

/// Represents a callback handler that will be shared with the underlying sqlite3 connection.
pub(crate) struct Handler(NonNull<dyn FnMut() -> bool + Send + 'static>);
unsafe impl Send for Handler {}

pub(crate) struct ConnectionState {
    pub(crate) handle: ConnectionHandle,

    // transaction status
    pub(crate) transaction_depth: usize,

    pub(crate) statements: Statements,

    log_settings: LogSettings,

    /// Stores the progress handler set on the current connection. If the handler returns `false`,
    /// the query is interrupted.
    progress_handler_callback: Option<Handler>,
}

impl ConnectionState {
    /// Drops the `progress_handler_callback` if it exists.
    pub(crate) fn remove_progress_handler(&mut self) {
        if let Some(mut handler) = self.progress_handler_callback.take() {
            unsafe {
                sqlite3_progress_handler(self.handle.as_ptr(), 0, None, 0 as *mut _);
                let _ = { Box::from_raw(handler.0.as_mut()) };
            }
        }
    }
}

pub(crate) struct Statements {
    // cache of semi-persistent statements
    cached: StatementCache<VirtualStatement>,
    // most recent non-persistent statement
    temp: Option<VirtualStatement>,
}

impl SqliteConnection {
    pub(crate) async fn establish(options: &SqliteConnectOptions) -> Result<Self, Error> {
        let params = EstablishParams::from_options(options)?;
        let worker = ConnectionWorker::establish(params).await?;
        Ok(Self {
            optimize_on_close: options.optimize_on_close.clone(),
            worker,
            row_channel_size: options.row_channel_size,
        })
    }

    /// Lock the SQLite database handle out from the worker thread so direct SQLite API calls can
    /// be made safely.
    ///
    /// Returns an error if the worker thread crashed.
    pub async fn lock_handle(&mut self) -> Result<LockedSqliteHandle<'_>, Error> {
        let guard = self.worker.unlock_db().await?;

        Ok(LockedSqliteHandle { guard })
    }
}

impl Debug for SqliteConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnection")
            .field("row_channel_size", &self.row_channel_size)
            .field("cached_statements_size", &self.cached_statements_size())
            .finish()
    }
}

impl Connection for SqliteConnection {
    type Database = Sqlite;

    type Options = SqliteConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            if let OptimizeOnClose::Enabled { analysis_limit } = self.optimize_on_close {
                let mut pragma_string = String::new();
                if let Some(limit) = analysis_limit {
                    write!(pragma_string, "PRAGMA analysis_limit = {limit}; ").ok();
                }
                pragma_string.push_str("PRAGMA optimize;");
                self.execute(&*pragma_string).await?;
            }
            let shutdown = self.worker.shutdown();
            // Drop the statement worker, which should
            // cover all references to the connection handle outside of the worker thread
            drop(self);
            // Ensure the worker thread has terminated
            shutdown.await
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            drop(self);
            Ok(())
        })
    }

    /// Ensure the background worker thread is alive and accepting commands.
    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.worker.ping())
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    fn cached_statements_size(&self) -> usize {
        self.worker
            .shared
            .cached_statements_size
            .load(std::sync::atomic::Ordering::Acquire)
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.worker.clear_cache().await?;
            Ok(())
        })
    }

    #[inline]
    fn shrink_buffers(&mut self) {
        // No-op.
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite, FLUSH does effectively nothing...
        // Well, we could use this to ensure that the command channel has been cleared,
        // but it would only develop a backlog if a lot of queries are executed and then cancelled
        // partway through, and then this would only make that situation worse.
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        false
    }
}

/// Implements a C binding to a progress callback. The function returns `0` if the
/// user-provided callback returns `true`, and `1` otherwise to signal an interrupt.
extern "C" fn progress_callback<F>(callback: *mut c_void) -> c_int
where
    F: FnMut() -> bool,
{
    unsafe {
        let r = catch_unwind(|| {
            let callback: *mut F = callback.cast::<F>();
            (*callback)()
        });
        c_int::from(!r.unwrap_or_default())
    }
}

impl LockedSqliteHandle<'_> {
    /// Returns the underlying sqlite3* connection handle.
    ///
    /// As long as this `LockedSqliteHandle` exists, it is guaranteed that the background thread
    /// is not making FFI calls on this database handle or any of its statements.
    ///
    /// ### Note: The `sqlite3` type is semver-exempt.
    /// This API exposes the `sqlite3` type from `libsqlite3-sys` crate for type safety.
    /// However, we reserve the right to upgrade `libsqlite3-sys` as necessary.
    ///
    /// Thus, if you are making direct calls via `libsqlite3-sys` you should pin the version
    /// of SQLx that you're using, and upgrade it and `libsqlite3-sys` manually as new
    /// versions are released.
    ///
    /// See [the driver root docs][crate] for details.
    pub fn as_raw_handle(&mut self) -> NonNull<sqlite3> {
        self.guard.handle.as_non_null_ptr()
    }

    /// Apply a collation to the open database.
    ///
    /// See [`SqliteConnectOptions::collation()`] for details.
    pub fn create_collation(
        &mut self,
        name: &str,
        compare: impl Fn(&str, &str) -> Ordering + Send + Sync + 'static,
    ) -> Result<(), Error> {
        collation::create_collation(&mut self.guard.handle, name, compare)
    }

    /// Sets a progress handler that is invoked periodically during long running calls. If the progress callback
    /// returns `false`, then the operation is interrupted.
    ///
    /// `num_ops` is the approximate number of [virtual machine instructions](https://www.sqlite.org/opcode.html)
    /// that are evaluated between successive invocations of the callback. If `num_ops` is less than one then the
    /// progress handler is disabled.
    ///
    /// Only a single progress handler may be defined at one time per database connection; setting a new progress
    /// handler cancels the old one.
    ///
    /// The progress handler callback must not do anything that will modify the database connection that invoked
    /// the progress handler. Note that sqlite3_prepare_v2() and sqlite3_step() both modify their database connections
    /// in this context.
    pub fn set_progress_handler<F>(&mut self, num_ops: i32, callback: F)
    where
        F: FnMut() -> bool + Send + 'static,
    {
        unsafe {
            let callback_boxed = Box::new(callback);
            // SAFETY: `Box::into_raw()` always returns a non-null pointer.
            let callback = NonNull::new_unchecked(Box::into_raw(callback_boxed));
            let handler = callback.as_ptr() as *mut _;
            self.guard.remove_progress_handler();
            self.guard.progress_handler_callback = Some(Handler(callback));

            sqlite3_progress_handler(
                self.as_raw_handle().as_mut(),
                num_ops,
                Some(progress_callback::<F>),
                handler,
            );
        }
    }

    /// Removes the progress handler on a database connection. The method does nothing if no handler was set.
    pub fn remove_progress_handler(&mut self) {
        self.guard.remove_progress_handler();
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        // explicitly drop statements before the connection handle is dropped
        self.statements.clear();
        self.remove_progress_handler();
    }
}

impl Statements {
    fn new(capacity: usize) -> Self {
        Statements {
            cached: StatementCache::new(capacity),
            temp: None,
        }
    }

    fn get(&mut self, query: &str, persistent: bool) -> Result<&mut VirtualStatement, Error> {
        if !persistent || !self.cached.is_enabled() {
            return Ok(self.temp.insert(VirtualStatement::new(query, false)?));
        }

        let exists = self.cached.contains_key(query);

        if !exists {
            let statement = VirtualStatement::new(query, true)?;
            self.cached.insert(query, statement);
        }

        let statement = self.cached.get_mut(query).unwrap();

        if exists {
            // as this statement has been executed before, we reset before continuing
            statement.reset()?;
        }

        Ok(statement)
    }

    fn len(&self) -> usize {
        self.cached.len()
    }

    fn clear(&mut self) {
        self.cached.clear();
        self.temp = None;
    }
}
