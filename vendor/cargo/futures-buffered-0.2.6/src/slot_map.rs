//! A SlotMap impl that uses a pre-allocated buffer to allow for pinned access.
//!
//! Implementation inspired by <https://github.com/orlp/slotmap>

use alloc::{boxed::Box, vec::Vec};
use core::{hint::unreachable_unchecked, pin::Pin};

pub(crate) struct SlotMap<F> {
    slots: Pin<Box<[Slot<F>]>>,
    free_head: usize,
    filled: usize,
}

// A slot, which represents storage for a value and a current version.
// Can be occupied or vacant.
enum Slot<F> {
    Occupied(F),
    NextFree(usize),
}

impl<F> SlotMap<F> {
    /// Constructs a new, empty [`SlotMap`] with the given capacity
    pub fn new(capacity: usize) -> Self {
        let slots: Vec<_> = (1..=capacity).map(Slot::NextFree).collect();

        Self {
            slots: slots.into_boxed_slice().into(),
            free_head: 0,
            filled: 0,
        }
    }

    /// Inserts a value given by `f` into the slot map.
    pub fn insert_with<Arg>(
        &mut self,
        arg: Arg,
        mut f: impl FnMut(Arg) -> F,
    ) -> Result<usize, Arg> {
        let key = self.free_head;
        let Some(mut slot) = self.get_slot(key) else {
            return Err(arg);
        };

        let Slot::NextFree(next_free) = *slot else {
            debug_assert!(false, "slotmap free_head pointed to a not free entry");
            unsafe { unreachable_unchecked() }
        };

        slot.set(Slot::Occupied(f(arg)));

        self.free_head = next_free;
        self.filled += 1;

        Ok(key)
    }

    /// Removes a key from the slot map
    pub fn remove(&mut self, key: usize) {
        let free_head = self.free_head;
        let Some(mut slot) = self.get_slot(key) else {
            return;
        };
        if let Slot::NextFree(_) = &*slot {
            return; // don't update if this slot is already free
        }
        slot.set(Slot::NextFree(free_head));
        self.free_head = key;
        self.filled -= 1;
    }

    fn get_slot(&mut self, key: usize) -> Option<Pin<&mut Slot<F>>> {
        // SAFETY: We return the inner data pinned and we never move the values within
        unsafe {
            let slots = self.slots.as_mut().get_unchecked_mut();
            let slot = slots.get_mut(key)?;
            Some(Pin::new_unchecked(slot))
        }
    }

    pub fn get(&mut self, key: usize) -> Option<Pin<&mut F>> {
        let slot = self.get_slot(key)?;
        // SAFETY: We return the inner data pinned and we never move the values within
        unsafe {
            match slot.get_unchecked_mut() {
                Slot::Occupied(f) => Some(Pin::new_unchecked(f)),
                Slot::NextFree(_) => None,
            }
        }
    }

    pub fn len(&self) -> usize {
        self.filled
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.filled == 0
    }

    pub fn iter_mut(&mut self) -> SlotMapIterMut<'_, F> {
        // SAFETY: Our iterator will return pinned values
        SlotMapIterMut(unsafe { self.slots.as_mut().get_unchecked_mut().iter_mut() })
    }
}

pub(crate) struct SlotMapIterMut<'a, F>(core::slice::IterMut<'a, Slot<F>>);
impl<'a, F> Iterator for SlotMapIterMut<'a, F> {
    type Item = Pin<&'a mut F>;

    fn next(&mut self) -> Option<Self::Item> {
        for f in self.0.by_ref() {
            if let Slot::Occupied(f) = f {
                // SAFETY: These values are guaranteed pinned
                return Some(unsafe { Pin::new_unchecked(f) });
            }
        }
        None
    }
}

impl<F> FromIterator<F> for SlotMap<F> {
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        // store the futures in our task list
        let inner: Box<[Slot<F>]> = iter.into_iter().map(Slot::Occupied).collect();

        // determine the actual capacity and create the shared state
        let cap = inner.len();

        // create the queue
        Self {
            slots: inner.into(),
            free_head: cap,
            filled: cap,
        }
    }
}
