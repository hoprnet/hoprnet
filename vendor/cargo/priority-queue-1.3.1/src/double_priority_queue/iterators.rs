/*
 *  Copyright 2017 Gianmarco Garrisi
 *
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version, or (at your opinion) under the terms
 *  of the Mozilla Public License version 2.0.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */
//! This module defines iterator types that are used only with the [`DoublePriorityQueue`]
//!
//! Usually you don't need to explicitly `use` any of the types declared here.

#[cfg(not(has_std))]
pub(crate) mod std {
    pub use core::*;
    pub mod alloc {
        pub use ::alloc::*;
    }
    pub mod collections {
        pub use ::alloc::collections::*;
    }
    pub use ::alloc::vec;
}

use std::cmp::{Eq, Ord};
#[cfg(has_std)]
use std::collections::hash_map::RandomState;
use std::hash::Hash;
use std::iter::*;

use crate::DoublePriorityQueue;

/// A mutable iterator over the couples `(item, priority)` of the `DoublePriorityQueue`
/// in arbitrary order.
///
/// It can be obtained calling the `iter_mut` method.
///
/// It can be used to update the priorities of the elements in the queue.
/// When the iterator goes out of scope, the heap is rebuilt to restore the
/// structural properties.
///
/// The item is mutable too, but it is a logical error to modify it in a way that
/// changes the result of any of `hash` or `eq`.
#[cfg(has_std)]
pub struct IterMut<'a, I: 'a, P: 'a, H: 'a = RandomState>
where
    I: Hash + Eq,
    P: Ord,
{
    pq: &'a mut DoublePriorityQueue<I, P, H>,
    pos: usize,
}

#[cfg(not(has_std))]
pub struct IterMut<'a, I: 'a, P: 'a, H: 'a>
where
    I: Hash + Eq,
    P: Ord,
{
    pq: &'a mut DoublePriorityQueue<I, P, H>,
    pos: usize,
}

impl<'a, I: 'a, P: 'a, H: 'a> IterMut<'a, I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) fn new(pq: &'a mut DoublePriorityQueue<I, P, H>) -> Self {
        IterMut { pq, pos: 0 }
    }
}

impl<'a, 'b: 'a, I: 'a, P: 'a, H: 'a> Iterator for IterMut<'a, I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = (&'a mut I, &'a mut P);
    fn next(&mut self) -> Option<Self::Item> {
        let r: Option<(&'a mut I, &'a mut P)> = self
            .pq
            .store
            .map
            .get_index_mut(self.pos)
            .map(|(i, p)| (i as *mut I, p as *mut P))
            .map(|(i, p)| unsafe { (i.as_mut().unwrap(), p.as_mut().unwrap()) });
        self.pos += 1;
        r
    }
}

impl<'a, I: 'a, P: 'a, H: 'a> Drop for IterMut<'a, I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    fn drop(&mut self) {
        self.pq.heap_build();
    }
}

/// A consuming iterator over the couples `(item, priority)` of the `PriorityQueue`
/// ordered by priority, from the lowest to the highest.
///
/// It can be obtained calling the `into_sorted_iter` method.
///
/// Since it implements [`DoubleEndedIterator`], this iterator can be reversed at any time
/// calling `rev`, at which point, elements will be extracted from the one with maximum priority
/// to the one with minimum priority.
#[cfg(has_std)]
pub struct IntoSortedIter<I, P, H = RandomState>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) pq: DoublePriorityQueue<I, P, H>,
}

#[cfg(not(has_std))]
pub struct IntoSortedIter<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) pq: DoublePriorityQueue<I, P, H>,
}

impl<I, P, H> Iterator for IntoSortedIter<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = (I, P);
    fn next(&mut self) -> Option<(I, P)> {
        self.pq.pop_min()
    }
}

impl<I, P, H> DoubleEndedIterator for IntoSortedIter<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    fn next_back(&mut self) -> Option<(I, P)> {
        self.pq.pop_max()
    }
}
