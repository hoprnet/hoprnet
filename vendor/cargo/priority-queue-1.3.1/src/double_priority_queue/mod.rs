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
//! This module contains the [`DoublePriorityQueue`] type and the related iterators.
//!
//! See the type level documentation for more details and examples.

pub mod iterators;

#[cfg(not(has_std))]
use std::vec::Vec;

use crate::core_iterators::{IntoIter, Iter};
use crate::store::{Index, Position, Store};
use iterators::*;

use std::borrow::Borrow;
use std::cmp::{Eq, Ord};
#[cfg(has_std)]
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::iter::{Extend, FromIterator, IntoIterator, Iterator};
use std::mem::replace;

/// A double priority queue with efficient change function to change the priority of an
/// element.
///
/// The priority is of type P, that must implement `std::cmp::Ord`.
///
/// The item is of type I, that must implement `Hash` and `Eq`.
///
/// Implemented as a heap of indexes, stores the items inside an `IndexMap`
/// to be able to retrieve them quickly.
///
/// With this data structure it is possible to efficiently extract both
/// the maximum and minimum elements arbitrarily.
///
/// If your need is to always extract the minimum, use a
/// `PriorityQueue<I, Reverse<P>>` wrapping
/// your priorities in the standard wrapper
/// [`Reverse<T>`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html).
///
///
/// # Example
/// ```rust
/// use priority_queue::DoublePriorityQueue;
///
/// let mut pq = DoublePriorityQueue::new();
///
/// assert!(pq.is_empty());
/// pq.push("Apples", 5);
/// pq.push("Bananas", 8);
/// pq.push("Strawberries", 23);
///
/// assert_eq!(pq.peek_max(), Some((&"Strawberries", &23)));
/// assert_eq!(pq.peek_min(), Some((&"Apples", &5)));
///
/// pq.change_priority("Bananas", 25);
/// assert_eq!(pq.peek_max(), Some((&"Bananas", &25)));
///
/// for (item, _) in pq.into_sorted_iter() {
///     println!("{}", item);
/// }
/// ```
#[derive(Clone)]
#[cfg(has_std)]
pub struct DoublePriorityQueue<I, P, H = RandomState>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) store: Store<I, P, H>,
}

#[derive(Clone)]
#[cfg(not(has_std))]
pub struct DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) store: Store<I, P, H>,
}

// do not [derive(Eq)] to loosen up trait requirements for other types and impls
impl<I, P, H> Eq for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
}

impl<I, P, H> Default for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_default_hasher()
    }
}

#[cfg(has_std)]
impl<I, P> DoublePriorityQueue<I, P>
where
    P: Ord,
    I: Hash + Eq,
{
    /// Creates an empty `DoublePriorityQueue`
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_default_hasher(capacity)
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher + Default,
{
    /// Creates an empty `DoublePriorityQueue` with the default hasher
    pub fn with_default_hasher() -> Self {
        Self::with_capacity_and_default_hasher(0)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and default hasher
    pub fn with_capacity_and_default_hasher(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, H::default())
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher,
{
    /// Creates an empty `DoublePriorityQueue` with the specified hasher
    pub fn with_hasher(hash_builder: H) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }

    /// Creates an empty `DoublePriorityQueue` with the specified capacity and hasher
    ///
    /// The internal collections will be able to hold at least `capacity`
    /// elements without reallocating.
    /// If `capacity` is 0, there will be no allocation.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: H) -> Self {
        Self {
            store: Store::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    /// Returns an iterator in arbitrary order over the
    /// (item, priority) elements in the queue
    pub fn iter(&self) -> Iter<I, P> {
        self.store.iter()
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
{
    /// Return an iterator in arbitrary order over the
    /// (item, priority) elements in the queue.
    ///
    /// The item and the priority are mutable references, but it's a logic error
    /// to modify the item in a way that change the result of `Hash` or `Eq`.
    ///
    /// It's *not* an error, instead, to modify the priorities, because the heap
    /// will be rebuilt once the `IterMut` goes out of scope. It would be
    /// rebuilt even if no priority value would have been modified, but the
    /// procedure will not move anything, but just compare the priorities.
    pub fn iter_mut(&mut self) -> IterMut<I, P, H> {
        IterMut::new(self)
    }

    /// Returns the couple (item, priority) with the lowest
    /// priority in the queue, or None if it is empty.
    ///
    /// Computes in **O(1)** time
    pub fn peek_min(&self) -> Option<(&I, &P)> {
        self.find_min().and_then(|i| {
            self.store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(i.0) }.0)
        })
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// Computes in **O(1)** time
    pub fn peek_max(&self) -> Option<(&I, &P)> {
        self.find_max().and_then(|i| {
            self.store
                .map
                .get_index(unsafe { *self.store.heap.get_unchecked(i.0) }.0)
        })
    }

    /// Returns the couple (item, priority) with the lowest
    /// priority in the queue, or None if it is empty.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    ///
    /// Computes in **O(1)** time
    pub fn peek_min_mut(&mut self) -> Option<(&mut I, &P)> {
        self.find_min()
            .and_then(move |i| {
                self.store
                    .map
                    .get_index_mut(unsafe { *self.store.heap.get_unchecked(i.0) }.0)
            })
            .map(|(k, v)| (k, &*v))
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    ///
    /// Computes in **O(1)** time
    pub fn peek_max_mut(&mut self) -> Option<(&mut I, &P)> {
        self.find_max()
            .and_then(move |i| {
                self.store
                    .map
                    .get_index_mut(unsafe { *self.store.heap.get_unchecked(i.0) }.0)
            })
            .map(|(k, v)| (k, &*v))
    }

    /// Returns the number of elements the internal map can hold without
    /// reallocating.
    ///
    /// This number is a lower bound; the map might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    pub fn capacity(&self) -> usize {
        self.store.capacity()
    }

    /// Shrinks the capacity of the internal data structures
    /// that support this operation as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.store.shrink_to_fit();
    }

    /// Removes the item with the lowest priority from
    /// the priority queue and returns the pair (item, priority),
    /// or None if the queue is empty.
    pub fn pop_min(&mut self) -> Option<(I, P)> {
        self.find_min().and_then(|i| {
            let r = self.store.swap_remove(i);
            self.heapify(i);
            r
        })
    }

    /// Removes the item with the greatest priority from
    /// the priority queue and returns the pair (item, priority),
    /// or None if the queue is empty.
    pub fn pop_max(&mut self) -> Option<(I, P)> {
        self.find_max().and_then(|i| {
            let r = self.store.swap_remove(i);
            self.heapify(i);
            r
        })
    }

    /// Implements a HeapSort.
    ///
    /// Consumes the PriorityQueue and returns a vector
    /// with all the items sorted from the one associated to
    /// the lowest priority to the highest.
    pub fn into_ascending_sorted_vec(mut self) -> Vec<I> {
        let mut res = Vec::with_capacity(self.store.size);
        while let Some((i, _)) = self.pop_min() {
            res.push(i);
        }
        res
    }

    /// Implements a HeapSort
    ///
    /// Consumes the PriorityQueue and returns a vector
    /// with all the items sorted from the one associated to
    /// the highest priority to the lowest.
    pub fn into_descending_sorted_vec(mut self) -> Vec<I> {
        let mut res = Vec::with_capacity(self.store.size);
        while let Some((i, _)) = self.pop_max() {
            res.push(i);
        }
        res
    }

    /// Returns the number of elements in the priority queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns true if the priority queue contains no elements.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Generates a new double ended iterator from self that
    /// will extract the elements from the one with the lowest priority
    /// to the highest one.
    pub fn into_sorted_iter(self) -> IntoSortedIter<I, P, H> {
        IntoSortedIter { pq: self }
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
    H: BuildHasher,
{
    // reserve_exact -> IndexMap does not implement reserve_exact

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `DoublePriorityQueue`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
    }

    /// Insert the item-priority pair into the queue.
    ///
    /// If an element equal to `item` was already into the queue,
    /// it is updated and the old value of its priority is returned in `Some`;
    /// otherwise, returns `None`.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push(&mut self, item: I, priority: P) -> Option<P> {
        use indexmap::map::Entry::*;
        let mut pos = Position(0);
        let mut oldp = None;

        match self.store.map.entry(item) {
            Occupied(mut e) => {
                oldp = Some(replace(e.get_mut(), priority));
                pos = unsafe { *self.store.qp.get_unchecked(e.index()) };
            }
            Vacant(e) => {
                e.insert(priority);
            }
        }

        if oldp.is_some() {
            self.up_heapify(pos);
            return oldp;
        }
        // get a reference to the priority
        // copy the current size of the heap
        let i = self.len();
        // add the new element in the qp vector as the last in the heap
        self.store.qp.push(Position(i));
        self.store.heap.push(Index(i));
        self.bubble_up(Position(i), Index(i));
        self.store.size += 1;
        None
    }

    /// Increase the priority of an existing item in the queue, or
    /// insert it if not present.
    ///
    /// If an element equal to `item` is already in the queue with a
    /// lower priority, its priority is increased to the new one
    /// without replacing the element and the old priority is returned.
    /// Otherwise, the new element is inserted into the queue.
    ///
    /// Returns `Some` if an element equal to `item` is already in the
    /// queue. If its priority is higher then `priority`, the latter is returned back,
    /// otherwise, the old priority is contained in the Option.
    /// If the item is not in the queue, `None` is returned.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push_increase(&mut self, item: I, priority: P) -> Option<P> {
        if self.get_priority(&item).map_or(true, |p| priority > *p) {
            self.push(item, priority)
        } else {
            Some(priority)
        }
    }

    /// Decrease the priority of an existing item in the queue, or
    /// insert it if not present.
    ///
    /// If an element equal to `item` is already in the queue with a
    /// higher priority, its priority is decreased to the new one
    /// without replacing the element and the old priority is returned.
    /// Otherwise, the new element is inserted into the queue.
    ///
    /// Returns `Some` if an element equal to `item` is already in the
    /// queue. If its priority is lower then `priority`, the latter is returned back,
    /// otherwise, the old priority is contained in the Option.
    /// If the item is not in the queue, `None` is returned.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push_decrease(&mut self, item: I, priority: P) -> Option<P> {
        if self.get_priority(&item).map_or(true, |p| priority < *p) {
            self.push(item, priority)
        } else {
            Some(priority)
        }
    }

    /// Change the priority of an Item returning the old value of priority,
    /// or `None` if the item wasn't in the queue.
    ///
    /// The argument `item` is only used for lookup, and is not used to overwrite the item's data
    /// in the priority queue.
    ///
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time.
    pub fn change_priority<Q: ?Sized>(&mut self, item: &Q, new_priority: P) -> Option<P>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store
            .change_priority(item, new_priority)
            .map(|(r, pos)| {
                self.up_heapify(pos);
                r
            })
    }

    /// Change the priority of an Item using the provided function.
    /// Return a boolean value where `true` means the item was in the queue and update was successful
    ///
    /// The argument `item` is only used for lookup, and is not used to overwrite the item's data
    /// in the priority queue.
    ///
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time (worst case).
    pub fn change_priority_by<Q: ?Sized, F>(&mut self, item: &Q, priority_setter: F) -> bool
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
        F: FnOnce(&mut P),
    {
        self.store
            .change_priority_by(item, priority_setter)
            .map(|pos| {
                self.up_heapify(pos);
            })
            .is_some()
    }

    /// Get the priority of an item, or `None`, if the item is not in the queue
    pub fn get_priority<Q: ?Sized>(&self, item: &Q) -> Option<&P>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get_priority(item)
    }

    /// Get the couple (item, priority) of an arbitrary element, as reference
    /// or `None` if the item is not in the queue.
    pub fn get<Q: ?Sized>(&self, item: &Q) -> Option<(&I, &P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get(item)
    }

    /// Get the couple (item, priority) of an arbitrary element, or `None`
    /// if the item was not in the queue.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    pub fn get_mut<Q: ?Sized>(&mut self, item: &Q) -> Option<(&mut I, &P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.get_mut(item)
    }

    /// Remove an arbitrary element from the priority queue.
    /// Returns the (item, priority) couple or None if the item
    /// is not found in the queue.
    ///
    /// The operation is performed in **O(log(N))** time (worst case).
    pub fn remove<Q: ?Sized>(&mut self, item: &Q) -> Option<(I, P)>
    where
        I: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.store.remove(item).map(|(item, priority, pos)| {
            if pos.0 < self.len() {
                self.up_heapify(pos);
            }

            (item, priority)
        })
    }

    /// Returns the items not ordered
    pub fn into_vec(self) -> Vec<I> {
        self.store.into_vec()
    }

    /// Drops all items from the priority queue
    pub fn clear(&mut self) {
        self.store.clear();
    }

    /// Move all items of the `other` queue to `self`
    /// ignoring the items Eq to elements already in `self`
    /// At the end, `other` will be empty.
    ///
    /// **Note** that at the end, the priority of the duplicated elements
    /// inside self may be the one of the elements in other,
    /// if other is longer than self
    pub fn append(&mut self, other: &mut Self) {
        self.store.append(&mut other.store);
        self.heap_build();
    }
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
{
}

impl<I, P, H> DoublePriorityQueue<I, P, H>
where
    P: Ord,
    I: Hash + Eq,
{
    /**************************************************************************/
    /*                            internal functions                          */

    fn heapify(&mut self, i: Position) {
        if self.len() <= 1 {
            return;
        }
        if level(i) % 2 == 0 {
            self.heapify_min(i)
        } else {
            self.heapify_max(i)
        }
    }

    fn heapify_min(&mut self, mut i: Position) {
        while i <= parent(Position(self.len() - 1)) {
            let m = i;

            let l = left(i);
            let r = right(i);
            // Minimum of childs and grandchilds
            i = *[l, r, left(l), right(l), left(r), right(r)]
                .iter()
                .map_while(|i| self.store.heap.get(i.0).map(|index| (i, index)))
                .min_by_key(|(_, index)| {
                    self.store
                        .map
                        .get_index(index.0)
                        .map(|(_, priority)| priority)
                        .unwrap()
                })
                .unwrap()
                .0;

            if unsafe {
                self.store.get_priority_from_position(i) < self.store.get_priority_from_position(m)
            } {
                self.store.swap(i, m);
                if i > r {
                    // i is a grandchild of m
                    let p = parent(i);
                    if unsafe {
                        self.store.get_priority_from_position(i)
                            > self.store.get_priority_from_position(p)
                    } {
                        self.store.swap(i, p);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn heapify_max(&mut self, mut i: Position) {
        while i <= parent(Position(self.len() - 1)) {
            let m = i;

            let l = left(i);
            let r = right(i);
            // Minimum of childs and grandchilds
            i = *[l, r, left(l), right(l), left(r), right(r)]
                .iter()
                .map_while(|i| self.store.heap.get(i.0).map(|index| (i, index)))
                .max_by_key(|(_, index)| {
                    self.store
                        .map
                        .get_index(index.0)
                        .map(|(_, priority)| priority)
                        .unwrap()
                })
                .unwrap()
                .0;

            if unsafe {
                self.store.get_priority_from_position(i) > self.store.get_priority_from_position(m)
            } {
                self.store.swap(i, m);
                if i > r {
                    // i is a grandchild of m
                    let p = parent(i);
                    if unsafe {
                        self.store.get_priority_from_position(i)
                            < self.store.get_priority_from_position(p)
                    } {
                        self.store.swap(i, p);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn bubble_up(&mut self, mut position: Position, map_position: Index) -> Position {
        let priority = self.store.map.get_index(map_position.0).unwrap().1;
        if position.0 > 0 {
            let parent = parent(position);
            let parent_priority = unsafe { self.store.get_priority_from_position(parent) };
            let parent_index = unsafe { *self.store.heap.get_unchecked(parent.0) };
            position = match (level(position) % 2 == 0, parent_priority < priority) {
                // on a min level and greater then parent
                (true, true) => {
                    unsafe {
                        *self.store.heap.get_unchecked_mut(position.0) = parent_index;
                        *self.store.qp.get_unchecked_mut(parent_index.0) = position;
                    }
                    self.bubble_up_max(parent, map_position)
                }
                // on a min level and less then parent
                (true, false) => self.bubble_up_min(position, map_position),
                // on a max level and greater then parent
                (false, true) => self.bubble_up_max(position, map_position),
                // on a max level and less then parent
                (false, false) => {
                    unsafe {
                        *self.store.heap.get_unchecked_mut(position.0) = parent_index;
                        *self.store.qp.get_unchecked_mut(parent_index.0) = position;
                    }
                    self.bubble_up_min(parent, map_position)
                }
            }
        }

        unsafe {
            // put the new element into the heap and
            // update the qp translation table and the size
            *self.store.heap.get_unchecked_mut(position.0) = map_position;
            *self.store.qp.get_unchecked_mut(map_position.0) = position;
        }
        position
    }

    fn bubble_up_min(&mut self, mut position: Position, map_position: Index) -> Position {
        let priority = self.store.map.get_index(map_position.0).unwrap().1;
        let mut grand_parent = Position(0);
        while if position.0 > 0 && parent(position).0 > 0 {
            grand_parent = parent(parent(position));
            (unsafe { self.store.get_priority_from_position(grand_parent) }) > priority
        } else {
            false
        } {
            unsafe {
                let grand_parent_index = *self.store.heap.get_unchecked(grand_parent.0);
                *self.store.heap.get_unchecked_mut(position.0) = grand_parent_index;
                *self.store.qp.get_unchecked_mut(grand_parent_index.0) = position;
            }
            position = grand_parent;
        }
        position
    }

    fn bubble_up_max(&mut self, mut position: Position, map_position: Index) -> Position {
        let priority = self.store.map.get_index(map_position.0).unwrap().1;
        let mut grand_parent = Position(0);
        while if position.0 > 0 && parent(position).0 > 0 {
            grand_parent = parent(parent(position));
            (unsafe { self.store.get_priority_from_position(grand_parent) }) < priority
        } else {
            false
        } {
            unsafe {
                let grand_parent_index = *self.store.heap.get_unchecked(grand_parent.0);
                *self.store.heap.get_unchecked_mut(position.0) = grand_parent_index;
                *self.store.qp.get_unchecked_mut(grand_parent_index.0) = position;
            }
            position = grand_parent;
        }
        position
    }

    fn up_heapify(&mut self, i: Position) {
        let tmp = unsafe { *self.store.heap.get_unchecked(i.0) };
        let pos = self.bubble_up(i, tmp);
        if i != pos {
            self.heapify(i)
        }
        self.heapify(pos);
    }

    /// Internal function that transform the `heap`
    /// vector in a heap with its properties
    ///
    /// Computes in **O(N)**
    pub(crate) fn heap_build(&mut self) {
        if self.len() == 0 {
            return;
        }
        for i in (0..=parent(Position(self.len())).0).rev() {
            self.heapify(Position(i));
        }
    }

    /// Returns the index of the max element
    fn find_max(&self) -> Option<Position> {
        match self.len() {
            0 => None,
            1 => Some(Position(0)),
            2 => Some(Position(1)),
            _ => Some(
                *[Position(1), Position(2)]
                    .iter()
                    .max_by_key(|i| unsafe { self.store.get_priority_from_position(**i) })
                    .unwrap(),
            ),
        }
    }

    /// Returns the index of the min element
    fn find_min(&self) -> Option<Position> {
        match self.len() {
            0 => None,
            _ => Some(Position(0)),
        }
    }
}

//FIXME: fails when the vector contains repeated items
// FIXED: repeated items ignored
impl<I, P, H> From<Vec<(I, P)>> for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn from(vec: Vec<(I, P)>) -> Self {
        let store = Store::from(vec);
        let mut pq = DoublePriorityQueue { store };
        pq.heap_build();
        pq
    }
}

use crate::PriorityQueue;

impl<I, P, H> From<PriorityQueue<I, P, H>> for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
    fn from(pq: PriorityQueue<I, P, H>) -> Self {
        let store = pq.store;
        let mut this = Self { store };
        this.heap_build();
        this
    }
}

//FIXME: fails when the iterator contains repeated items
// FIXED: the item inside the pq is updated
// so there are two functions with different behaviours.
impl<I, P, H> FromIterator<(I, P)> for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher + Default,
{
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = (I, P)>,
    {
        let store = Store::from_iter(iter);
        let mut pq = DoublePriorityQueue { store };
        pq.heap_build();
        pq
    }
}

impl<I, P, H> IntoIterator for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
    type Item = (I, P);
    type IntoIter = IntoIter<I, P>;
    fn into_iter(self) -> IntoIter<I, P> {
        self.store.into_iter()
    }
}

impl<'a, I, P, H> IntoIterator for &'a DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
    type Item = (&'a I, &'a P);
    type IntoIter = Iter<'a, I, P>;
    fn into_iter(self) -> Iter<'a, I, P> {
        self.store.iter()
    }
}

impl<'a, I, P, H> IntoIterator for &'a mut DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = (&'a mut I, &'a mut P);
    type IntoIter = IterMut<'a, I, P, H>;
    fn into_iter(self) -> IterMut<'a, I, P, H> {
        IterMut::new(self)
    }
}

impl<I, P, H> Extend<(I, P)> for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq,
    P: Ord,
    H: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (I, P)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let rebuild = if let Some(max) = max {
            self.reserve(max);
            better_to_rebuild(self.len(), max)
        } else if min != 0 {
            self.reserve(min);
            better_to_rebuild(self.len(), min)
        } else {
            false
        };
        if rebuild {
            self.store.extend(iter);
            self.heap_build();
        } else {
            for (item, priority) in iter {
                self.push(item, priority);
            }
        }
    }
}

use std::fmt;

impl<I, P, H> fmt::Debug for DoublePriorityQueue<I, P, H>
where
    I: Hash + Eq + fmt::Debug,
    P: Ord + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.store.fmt(f)
    }
}

use std::cmp::PartialEq;

impl<I, P1, H1, P2, H2> PartialEq<DoublePriorityQueue<I, P2, H2>> for DoublePriorityQueue<I, P1, H1>
where
    I: Hash + Eq,
    P1: Ord,
    P1: PartialEq<P2>,
    Option<P1>: PartialEq<Option<P2>>,
    P2: Ord,
    H1: BuildHasher,
    H2: BuildHasher,
{
    fn eq(&self, other: &DoublePriorityQueue<I, P2, H2>) -> bool {
        self.store == other.store
    }
}

/// Compute the index of the left child of an item from its index
#[inline(always)]
const fn left(i: Position) -> Position {
    Position((i.0 * 2) + 1)
}
/// Compute the index of the right child of an item from its index
#[inline(always)]
const fn right(i: Position) -> Position {
    Position((i.0 * 2) + 2)
}
/// Compute the index of the parent element in the heap from its index
#[inline(always)]
const fn parent(i: Position) -> Position {
    Position((i.0 - 1) / 2)
}

// Compute the level of a node from its index
#[inline(always)]
const fn level(i: Position) -> usize {
    log2_fast(i.0 + 1)
}

#[inline(always)]
const fn log2_fast(x: usize) -> usize {
    (8 * usize::BITS - x.leading_zeros() - 1) as usize
}

// `rebuild` takes O(len1 + len2) operations
// and about 2 * (len1 + len2) comparisons in the worst case
// while `extend` takes O(len2 * log_2(len1)) operations
// and about 1 * len2 * log_2(len1) comparisons in the worst case,
// assuming len1 >= len2.
fn better_to_rebuild(len1: usize, len2: usize) -> bool {
    // log(1) == 0, so the inequation always falsy
    // log(0) is inapplicable and produces panic
    if len1 <= 1 {
        return false;
    }

    2 * (len1 + len2) < len2 * log2_fast(len1)
}

#[cfg(feature = "serde")]
mod serde {
    use std::cmp::{Eq, Ord};
    use std::hash::{BuildHasher, Hash};

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    use super::DoublePriorityQueue;
    use crate::store::Store;

    impl<I, P, H> Serialize for DoublePriorityQueue<I, P, H>
    where
        I: Hash + Eq + Serialize,
        P: Ord + Serialize,
        H: BuildHasher,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.store.serialize(serializer)
        }
    }

    impl<'de, I, P, H> Deserialize<'de> for DoublePriorityQueue<I, P, H>
    where
        I: Hash + Eq + Deserialize<'de>,
        P: Ord + Deserialize<'de>,
        H: BuildHasher + Default,
    {
        fn deserialize<D>(deserializer: D) -> Result<DoublePriorityQueue<I, P, H>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Store::deserialize(deserializer).map(|store| {
                let mut pq = DoublePriorityQueue { store };
                pq.heap_build();
                pq
            })
        }
    }
}
