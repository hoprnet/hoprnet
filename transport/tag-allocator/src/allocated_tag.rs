use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::stack::ArrayStack;

/// A tag value that is automatically returned to its allocator on drop.
///
/// Not `Copy` or `Clone` — each tag has exactly one owner.
pub struct AllocatedTag {
    value: u64,
    pool: Arc<ArrayStack>,
}

impl AllocatedTag {
    pub(crate) fn new(value: u64, pool: Arc<ArrayStack>) -> Self {
        Self { value, pool }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Drop for AllocatedTag {
    fn drop(&mut self) {
        // Best-effort return; if the pool is full the tag is simply lost.
        let _ = self.pool.push(self.value);
    }
}

impl From<&AllocatedTag> for u64 {
    fn from(tag: &AllocatedTag) -> u64 {
        tag.value
    }
}

impl PartialEq<u64> for AllocatedTag {
    fn eq(&self, other: &u64) -> bool {
        self.value == *other
    }
}

impl PartialEq for AllocatedTag {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AllocatedTag {}

impl Hash for AllocatedTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl fmt::Display for AllocatedTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for AllocatedTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllocatedTag({})", self.value)
    }
}
