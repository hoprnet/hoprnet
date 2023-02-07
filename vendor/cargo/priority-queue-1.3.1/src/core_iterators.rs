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
//! This module defines iterator types that are used with
//! both the [`PriorityQueue`](super::PriorityQueue) and the [`DoublePriorityQueue`](super::DoublePriorityQueue)
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

use std::hash::Hash;

/// An iterator in arbitrary order over the couples
/// `(item, priority)` in the queue.
///
/// It can be obtained calling the `iter` method.
pub struct Iter<'a, I: 'a, P: 'a>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) iter: ::indexmap::map::Iter<'a, I, P>,
}

impl<'a, I: 'a, P: 'a> Iterator for Iter<'a, I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = (&'a I, &'a P);
    fn next(&mut self) -> Option<(&'a I, &'a P)> {
        self.iter.next()
    }
}

/// An iterator in arbitrary order over the couples
/// `(item, priority)` that consumes the queue.
///
/// It can be obtained calling the `into_iter` method from the `IntoIterator` trait.
pub struct IntoIter<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    pub(crate) iter: ::indexmap::map::IntoIter<I, P>,
}

impl<I, P> Iterator for IntoIter<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = (I, P);
    fn next(&mut self) -> Option<(I, P)> {
        self.iter.next()
    }
}
