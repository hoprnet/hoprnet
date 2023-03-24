use super::SMA;
use crate::{common::cast_to_divisor_type, ring_buffer::RingBuffer, Iter};
use num_traits::{FromPrimitive, Zero};
use std::{
	marker::{self, PhantomData},
	ops::{AddAssign, Div, SubAssign},
};

/// An SMA implementation that caches the sum of all samples currently in the sample window as a
/// single value.
pub struct SingleSumSMA<Sample, Divisor, const WINDOW_SIZE: usize> {
	samples: RingBuffer<Sample, WINDOW_SIZE>,
	sum: Sample,
	_marker: marker::PhantomData<Divisor>,
}

impl<Sample, Divisor, const WINDOW_SIZE: usize> SMA<Sample, Divisor, WINDOW_SIZE>
	for SingleSumSMA<Sample, Divisor, WINDOW_SIZE>
where
	Sample: Copy + AddAssign + SubAssign + Div<Divisor, Output = Sample>,
	Divisor: FromPrimitive,
{
	fn add_sample(&mut self, new_sample: Sample) {
		if WINDOW_SIZE == 0 {
			return;
		}

		self.sum += new_sample;

		if let Some(shifted_sample) = self.samples.shift(new_sample) {
			self.sum -= shifted_sample;
		}
	}

	fn get_average(&self) -> Sample {
		let num_samples = self.samples.len();

		if num_samples == 0 {
			return self.sum;
		}

		self.sum / cast_to_divisor_type(num_samples)
	}

	fn get_most_recent_sample(&self) -> Option<Sample> {
		self.samples.front().cloned()
	}

	fn get_num_samples(&self) -> usize {
		self.samples.len()
	}

	fn get_sample_window_size(&self) -> usize {
		WINDOW_SIZE
	}

	fn get_sample_window_iter(&self) -> Iter<Sample, WINDOW_SIZE> {
		self.samples.iter()
	}
}

impl<Sample: Copy + Zero, Divisor, const WINDOW_SIZE: usize>
	SingleSumSMA<Sample, Divisor, WINDOW_SIZE>
{
	/// Constructs a new [SingleSumSMA] with window size `WINDOW_SIZE`. This constructor is
	/// only available for `Sample` types that implement [num_traits::Zero]. If the `Sample` type
	/// does not, use the [from_zero](SingleSumSMA::from_zero) constructor instead.
	///
	/// Note that the `Divisor` type usually cannot be derived by the compiler when using this
	/// constructor and must be explicitly stated, even if it is the same as the `Sample` type.
	pub fn new() -> Self {
		Self {
			samples: RingBuffer::new(Sample::zero()),
			sum: Sample::zero(),
			_marker: PhantomData,
		}
	}
}

impl<Sample: Copy, Divisor, const WINDOW_SIZE: usize> SingleSumSMA<Sample, Divisor, WINDOW_SIZE> {
	/// Constructs a new [SingleSumSMA] with window size `WINDOW_SIZE` from the given
	/// `zero` sample. If the `Sample` type implements [num_traits::Zero], the
	/// [new](SingleSumSMA::new) constructor might be preferable to this.
	pub fn from_zero(zero: Sample) -> Self {
		Self {
			samples: RingBuffer::new(zero),
			sum: zero,
			_marker: PhantomData,
		}
	}
}
