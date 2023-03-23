use super::{sum_tree::SumTree, SMA};
use crate::{common::cast_to_divisor_type, ring_buffer::RingBuffer, Iter};
use num_traits::{FromPrimitive, Zero};
use std::{
	marker::{self, PhantomData},
	ops::{Add, Div},
};

type SumTreeNodeIdx = usize;

/// An SMA implementation that caches the sum of all samples currently in the sample window as a
/// tree of sums.
pub struct SumTreeSMA<Sample, Divisor, const WINDOW_SIZE: usize> {
	samples: RingBuffer<SumTreeNodeIdx, WINDOW_SIZE>,
	sum_tree: SumTree<Sample>,
	_marker: marker::PhantomData<Divisor>,
}

impl<Sample, Divisor, const WINDOW_SIZE: usize> SMA<Sample, Divisor, WINDOW_SIZE>
	for SumTreeSMA<Sample, Divisor, WINDOW_SIZE>
where
	Sample: Copy + Add<Output = Sample> + Div<Divisor, Output = Sample>,
	Divisor: FromPrimitive,
{
	fn add_sample(&mut self, new_sample: Sample) {
		if WINDOW_SIZE == 0 {
			return;
		}

		let tree_node_idx = if self.samples.len() < WINDOW_SIZE {
			self.samples.len()
		} else {
			self.samples.pop_back().unwrap()
		};

		self.samples.push_front(tree_node_idx);

		self.sum_tree
			.update_leaf_node_sample(tree_node_idx, new_sample);
	}

	fn get_average(&self) -> Sample {
		let num_samples = self.samples.len();

		if num_samples == 0 {
			return self.sum_tree.get_root_sum();
		}

		self.sum_tree.get_root_sum() / cast_to_divisor_type(num_samples)
	}

	fn get_most_recent_sample(&self) -> Option<Sample> {
		self.samples
			.front()
			.map(|node_idx| self.sum_tree.get_leaf_node_sum(node_idx))
	}

	fn get_num_samples(&self) -> usize {
		self.samples.len()
	}

	fn get_sample_window_size(&self) -> usize {
		WINDOW_SIZE
	}

	fn get_sample_window_iter(&self) -> Iter<Sample, WINDOW_SIZE> {
		let num_samples = self.get_num_samples();
		Iter::new(
			self.sum_tree.get_leaf_nodes(num_samples),
			num_samples,
			num_samples,
		)
	}
}

impl<Sample: Copy + Zero, Divisor, const WINDOW_SIZE: usize>
	SumTreeSMA<Sample, Divisor, WINDOW_SIZE>
{
	/// Constructs a new [SumTreeSMA] with window size `WINDOW_SIZE`. This constructor is
	/// only available for `Sample` types that implement [num_traits::Zero]. If the `Sample` type
	/// does not, use the [from_zero](SumTreeSMA::from_zero) constructor instead.
	///
	/// Note that the `Divisor` type usually cannot be derived by the compiler when using this
	/// constructor and must be explicitly stated, even if it is the same as the `Sample` type.
	pub fn new() -> Self {
		Self {
			samples: RingBuffer::new(0),
			sum_tree: SumTree::new(Sample::zero(), WINDOW_SIZE),
			_marker: PhantomData,
		}
	}
}

impl<Sample: Copy, Divisor, const WINDOW_SIZE: usize> SumTreeSMA<Sample, Divisor, WINDOW_SIZE> {
	/// Constructs a new [SumTreeSMA] with window size `WINDOW_SIZE` from the given
	/// `zero` sample. If the `Sample` type implements [num_traits::Zero], the
	/// [new](SumTreeSMA::new) constructor might be preferable to this.
	pub fn from_zero(zero: Sample) -> Self {
		Self {
			samples: RingBuffer::new(0),
			sum_tree: SumTree::new(zero, WINDOW_SIZE),
			_marker: PhantomData,
		}
	}
}
