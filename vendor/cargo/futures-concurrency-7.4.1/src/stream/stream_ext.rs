use crate::stream::{IntoStream, Merge};
use futures_core::Stream;

use super::{chain::tuple::Chain2, merge::tuple::Merge2, zip::tuple::Zip2, Chain, Zip};

/// An extension trait for the `Stream` trait.
pub trait StreamExt: Stream {
    /// Combines two streams into a single stream of all their outputs.
    fn merge<T, S2>(self, other: S2) -> Merge2<T, Self, S2::IntoStream>
    where
        Self: Stream<Item = T> + Sized,
        S2: IntoStream<Item = T>;

    /// Takes two streams and creates a new stream over all in sequence
    fn chain<T, S2>(self, other: S2) -> Chain2<Self, S2::IntoStream>
    where
        Self: Stream<Item = T> + Sized,
        S2: IntoStream<Item = T>;

    /// ‘Zips up’ multiple streams into a single stream of pairs.
    fn zip<T, S2>(self, other: S2) -> Zip2<Self, S2::IntoStream>
    where
        Self: Stream<Item = T> + Sized,
        S2: IntoStream<Item = T>;
}

impl<S1> StreamExt for S1
where
    S1: Stream,
{
    fn merge<T, S2>(self, other: S2) -> Merge2<T, S1, S2::IntoStream>
    where
        S1: Stream<Item = T>,
        S2: IntoStream<Item = T>,
    {
        Merge::merge((self, other))
    }

    fn chain<T, S2>(self, other: S2) -> Chain2<Self, S2::IntoStream>
    where
        Self: Stream<Item = T> + Sized,
        S2: IntoStream<Item = T>,
    {
        // TODO(yosh): fix the bounds on the tuple impl
        Chain::chain((self, other.into_stream()))
    }

    fn zip<T, S2>(self, other: S2) -> Zip2<Self, S2::IntoStream>
    where
        Self: Stream<Item = T> + Sized,
        S2: IntoStream<Item = T>,
    {
        // TODO(yosh): fix the bounds on the tuple impl
        Zip::zip((self, other.into_stream()))
    }
}
