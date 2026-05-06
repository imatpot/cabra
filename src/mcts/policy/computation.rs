use std::time::{Duration, Instant};

/// A computational limit based on a fixed number of iterations.
pub struct Iterative {
	pub iterations: u32,
}

/// A computational limit based on a fixed duration of time.
pub struct Temporal {
	pub duration: Duration,
}

/// A trait representing a computational limit, allowing execution of a callback
/// until the limit is exhausted.
pub trait ComputationalLimit {
	/// Returns a predicate that returns `true` until the limit is exhausted.
	fn predicate(&self) -> Box<dyn FnMut() -> bool>;
}

impl Default for Box<dyn ComputationalLimit> {
	fn default() -> Self {
		Box::new(Iterative { iterations: 10_000 })
	}
}

impl ComputationalLimit for Iterative {
	fn predicate(&self) -> Box<dyn FnMut() -> bool> {
		let mut remaining = self.iterations;

		Box::new(move || {
			if remaining == 0 {
				return false;
			}
			remaining -= 1;
			true
		})
	}
}

impl ComputationalLimit for Temporal {
	fn predicate(&self) -> Box<dyn FnMut() -> bool> {
		let deadline = Instant::now() + self.duration;
		Box::new(move || Instant::now() < deadline)
	}
}
