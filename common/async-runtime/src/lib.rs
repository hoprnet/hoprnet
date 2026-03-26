//! Executor API for HOPR which exposes the necessary async functions depending on the enabled
//! runtime.

use std::hash::Hash;

pub use futures::future::AbortHandle;

// Both features could be enabled during testing; therefore, we only use tokio when it's
// exclusively enabled.
pub mod prelude {
    #[cfg(feature = "async-lock")]
    pub use async_lock::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
    pub use futures::future::{AbortHandle, abortable};
    #[cfg(all(feature = "runtime-tokio", not(feature = "async-lock")))]
    pub use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
    #[cfg(feature = "runtime-tokio")]
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

impl<T: std::fmt::Debug> std::fmt::Debug for AbortableList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.keys()).finish()
    }
}

impl<T: Hash + Eq> AbortableList<T> {
    /// Appends a new [`abortable task`](Abortable) to the end of this list.
    pub fn insert<A: Abortable + Send + Sync + 'static>(&mut self, process: T, task: A) {
        self.0.insert(process, Box::new(task));
    }

    /// Checks if the list contains a task with the given key.
    pub fn contains(&self, process: &T) -> bool {
        self.0.contains_key(process)
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

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::*;

    #[derive(Default)]
    struct MockTask {
        aborted: AtomicBool,
    }

    impl Abortable for MockTask {
        fn abort_task(&self) {
            self.aborted.store(true, Ordering::SeqCst);
        }

        fn was_aborted(&self) -> bool {
            self.aborted.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_insert_and_contains() {
        let mut list = AbortableList::default();
        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        list.insert("task1", task1.clone());
        list.insert("task2", task2.clone());

        assert!(list.contains(&"task1"));
        assert!(list.contains(&"task2"));
        assert!(!list.contains(&"task3"));
        assert_eq!(list.size(), 2);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_abort_one() {
        let mut list = AbortableList::default();
        let task1 = Arc::new(MockTask::default());

        list.insert("task1", task1.clone());
        assert!(list.abort_one(&"task1"));
        assert!(task1.was_aborted());
        assert!(!list.contains(&"task1"));
        assert_eq!(list.size(), 0);

        // Aborting already removed task
        assert!(!list.abort_one(&"task1"));
    }

    #[test]
    fn test_abort_one_already_aborted() {
        let mut list = AbortableList::default();
        let task1 = Arc::new(MockTask::default());
        task1.abort_task();

        list.insert("task1", task1.clone());
        // abort_one returns false if already aborted
        assert!(!list.abort_one(&"task1"));
        // Check that it was still removed from the list even if already aborted
        assert!(!list.contains(&"task1"));
    }

    #[test]
    fn test_debug_impl() {
        let mut list = AbortableList::default();
        list.insert("task1", MockTask::default());
        list.insert("task2", MockTask::default());
        let debug_str = format!("{:?}", list);
        assert!(debug_str.contains("task1"));
        assert!(debug_str.contains("task2"));
    }

    #[test]
    fn test_abort_all() {
        let mut list = AbortableList::default();
        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        list.insert(1, task1.clone());
        list.insert(2, task2.clone());

        list.abort_all();

        assert!(task1.was_aborted());
        assert!(task2.was_aborted());
        // abort_all doesn't remove from list
        assert_eq!(list.size(), 2);
    }

    #[test]
    fn test_drop_aborts_all() {
        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        {
            let mut list = AbortableList::default();
            list.insert(1, task1.clone());
            list.insert(2, task2.clone());
        }

        assert!(task1.was_aborted());
        assert!(task2.was_aborted());
    }

    #[test]
    fn test_extend_from() {
        let mut list1 = AbortableList::default();
        let mut list2 = AbortableList::default();

        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        list1.insert(1, task1.clone());
        list2.insert(2, task2.clone());

        list1.extend_from(list2);

        assert_eq!(list1.size(), 2);
        assert!(list1.contains(&1));
        assert!(list1.contains(&2));

        // Ensure task2 was not aborted during extend
        assert!(!task2.was_aborted());
    }

    #[test]
    fn test_flat_map_extend_from() {
        let mut list1 = AbortableList::default();
        let mut list2 = AbortableList::default();

        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        list1.insert("a", task1.clone());
        list2.insert(1, task2.clone());

        list1.flat_map_extend_from(list2, |k| if k == 1 { "b" } else { "c" });

        assert_eq!(list1.size(), 2);
        assert!(list1.contains(&"a"));
        assert!(list1.contains(&"b"));
    }

    #[test]
    fn test_nested_abortable_list() {
        let mut outer = AbortableList::default();
        let mut inner = AbortableList::default();

        let task1 = Arc::new(MockTask::default());
        inner.insert(1, task1.clone());

        outer.insert("inner", inner);

        outer.abort_all();
        assert!(task1.was_aborted());
    }

    #[test]
    fn test_was_aborted_all() {
        let mut list = AbortableList::default();
        let task1 = Arc::new(MockTask::default());
        let task2 = Arc::new(MockTask::default());

        list.insert(1, task1.clone());
        list.insert(2, task2.clone());

        assert!(!list.was_aborted());

        task1.abort_task();
        assert!(!list.was_aborted());

        task2.abort_task();
        assert!(list.was_aborted());
    }

    #[test]
    fn test_iter_names() {
        let mut list = AbortableList::default();
        list.insert("a", MockTask::default());
        list.insert("b", MockTask::default());
        list.insert("c", MockTask::default());

        let names: Vec<&&str> = list.iter_names().collect();
        assert_eq!(names, vec![&"a", &"b", &"c"]);
    }

    #[test]
    fn test_reverse_insertion_order_on_abort() {
        use std::sync::Mutex;
        let abort_order = Arc::new(Mutex::new(Vec::new()));

        struct OrderedMockTask {
            id: i32,
            order: Arc<Mutex<Vec<i32>>>,
        }

        impl Abortable for OrderedMockTask {
            fn abort_task(&self) {
                self.order.lock().unwrap().push(self.id);
            }

            fn was_aborted(&self) -> bool {
                self.order.lock().unwrap().contains(&self.id)
            }
        }

        let mut list = AbortableList::default();
        list.insert(
            1,
            OrderedMockTask {
                id: 1,
                order: abort_order.clone(),
            },
        );
        list.insert(
            2,
            OrderedMockTask {
                id: 2,
                order: abort_order.clone(),
            },
        );
        list.insert(
            3,
            OrderedMockTask {
                id: 3,
                order: abort_order.clone(),
            },
        );

        list.abort_all();

        let order = abort_order.lock().unwrap();
        assert_eq!(*order, vec![3, 2, 1]);
    }
}
