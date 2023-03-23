use crate::Iter;

/// This trait provides an common interface for algorithms that can calculate a simple moving
/// average.
///
/// In this crate, a simple moving average is defined as `sum(window(samples, N)) / length(window(samples, N))`.
/// Here `samples` is a possibly infinite series of samples. The `window` function extracts the last
/// `N` of those samples.
///
/// *Implementation detail:* For the purposes of this library, there is no point in keeping samples
/// outside the sample window around, so they are discarded when newer samples push them out of the
/// window. This allows the implementations to have constant memory requirements and be stack
/// allocated.
///
/// Terminology:
///  - Sample: A data point, a value.
///  - Sample window: The subset of all samples used for average calculations.
pub trait SMA<Sample, Divisor, const WINDOW_SIZE: usize> {
	/// Adds a sample to the series of samples. If the sample window is full, this will cause the
	/// oldest sample to be dropped, i.e. no longer contribute to the average.
	fn add_sample(&mut self, new_sample: Sample);

	/// Returns the simple moving average value of all the samples in the sample window.
	fn get_average(&self) -> Sample;

	/// Returns the total number of samples currently in the in the sample window. This value never
	/// exceeds the sample window size.
	fn get_num_samples(&self) -> usize;

	/// Returns the maximum number of samples that fit in the sample window.
	fn get_sample_window_size(&self) -> usize;

	// Returns an iterator over the samples currently in the sample window.
	fn get_sample_window_iter(&self) -> Iter<Sample, WINDOW_SIZE>;

	/// Returns the most recently added sample, if any.
	fn get_most_recent_sample(&self) -> Option<Sample>;
}
