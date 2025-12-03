//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.

use std::hash::Hash;

pub use futures::future::AbortHandle;

// Both features could be enabled during testing; therefore, we only use tokio when it's
// exclusively enabled.
#[cfg(feature = "runtime-tokio")]
pub mod prelude {
    pub use futures::future::{AbortHandle, abortable};
    pub use tokio::{
        task::{JoinError, JoinHandle, spawn, spawn_blocking, spawn_local},
        time::{sleep, timeout as timeout_fut},
    };
}

#[macro_export]
macro_rules! spawn_as_abortable {
    ($($expr:expr),*) => {{
        let (proc, abort_handle) = $crate::prelude::abortable($($expr),*);
        let _jh = $crate::prelude::spawn(proc);
        abort_handle
    }}
}

// If no runtime is enabled, fail compilation
#[cfg(not(feature = "runtime-tokio"))]
compile_error!("No runtime enabled");

/// Abstraction over tasks that can be aborted (such as join or abort handles).
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait Abortable {
    /// Notifies the task that it should abort.
    ///
    /// Must be idempotent and not panic if it was already called before, due to implementation-specific
    /// semantics of [`Abortable::was_aborted`].
    fn abort_task(&self);

    /// Returns `true` if [`abort_task`](Abortable::abort_task) was already called or the task has finished.
    ///
    /// It is implementation-specific whether `true` actually means that the task has been finished.
    /// The [`Abortable::abort_task`] therefore can be also called if `true` is returned without a consequence.
    fn was_aborted(&self) -> bool;
}

impl Abortable for AbortHandle {
    fn abort_task(&self) {
        self.abort();
    }

    fn was_aborted(&self) -> bool {
        self.is_aborted()
    }
}

#[cfg(feature = "runtime-tokio")]
impl Abortable for tokio::task::JoinHandle<()> {
    fn abort_task(&self) {
        self.abort();
    }

    fn was_aborted(&self) -> bool {
        self.is_finished()
    }
}

/// List of [`Abortable`] tasks with each task identified by a unique key of type `T`.
///
/// Abortable objects, such as join or abort handles, do not by design abort when dropped.
/// Sometimes this behavior is not desirable, and spawned run-away tasks may still continue to live
/// e.g.: after an error is raised.
///
/// This object allows safely managing abortable tasks and will terminate all the tasks in reverse insertion order once
/// dropped.
///
/// Additionally, this object also implements [`Abortable`] allowing it to be arbitrarily nested.
pub struct AbortableList<T>(indexmap::IndexMap<T, Box<dyn Abortable + Send + Sync>>);

impl<T> Default for AbortableList<T> {
    fn default() -> Self {
        Self(indexmap::IndexMap::new())
    }
}

impl<T: Hash + Eq> AbortableList<T> {
    /// Appends a new [`abortable task`](Abortable) to the end of this list.
    pub fn insert<A: Abortable + Send + Sync + 'static>(&mut self, process: T, task: A) {
        self.0.insert(process, Box::new(task));
    }

    /// Looks up a task by its key, removes it and aborts it.
    ///
    /// Returns `true` if the task was aborted and removed.
    /// Otherwise, returns `false` (including a situation when the task was present but already aborted).
    pub fn abort_one(&mut self, process: &T) -> bool {
        if let Some(item) = self.0.shift_remove(process).filter(|t| !t.was_aborted()) {
            item.abort_task();
            true
        } else {
            false
        }
    }

    /// Extends this list by appending `other`.
    ///
    /// The tasks from `other` are moved to this list without aborting them.
    /// Afterward, `other` will be empty.
    pub fn extend_from(&mut self, mut other: AbortableList<T>) {
        self.0.extend(other.0.drain(..));
    }

    /// Extends this list by appending `other` while mapping its keys to the ones in this list.
    ///
    /// The tasks from `other` are moved to this list without aborting them.
    /// Afterward, `other` will be empty.
    pub fn flat_map_extend_from<U>(&mut self, mut other: AbortableList<U>, key_map: impl Fn(U) -> T) {
        self.0.extend(other.0.drain(..).map(|(k, v)| (key_map(k), v)));
    }
}
impl<T> AbortableList<T> {
    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of abortable tasks in the list.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the task names in the insertion order.
    pub fn iter_names(&self) -> impl Iterator<Item = &T> {
        self.0.keys()
    }

    /// Aborts all tasks in this list in the reverse insertion order.
    ///
    /// Skips tasks which were [already aborted](Abortable::was_aborted).
    pub fn abort_all(&self) {
        for (_, task) in self.0.iter().rev().filter(|(_, task)| !task.was_aborted()) {
            task.abort_task();
        }
    }
}

impl<T> Abortable for AbortableList<T> {
    fn abort_task(&self) {
        self.abort_all();
    }

    fn was_aborted(&self) -> bool {
        self.0.iter().all(|(_, task)| task.was_aborted())
    }
}

impl<T> Drop for AbortableList<T> {
    fn drop(&mut self) {
        self.abort_all();
        self.0.clear();
    }
}
