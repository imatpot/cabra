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
pub trait ComputationalLimit: Send + Sync {
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

/// Defines the intensity of rollouts to perform during a rollout phase, which
/// can be used to scale the computational effort spent on rollouts compared to
/// tree traversal and selection.
pub struct ComputationalIntensity {
	/// The number of rollouts to perform per node during the rollout phase.
	pub rollouts_per_node: u8,

	/// The number of parallel trees to search during the rollout phase.
	/// Each tree will perform its own search, and before determining the best
	/// move, the results of all trees will be merged.
	pub trees: u8,
}

impl Default for ComputationalIntensity {
	fn default() -> Self {
		Self {
			rollouts_per_node: 1,
			trees: 1,
		}
	}
}
