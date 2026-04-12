use rand::{Rng, seq::IndexedRandom};

use crate::caminos::state::{GameResult, GameState};

/// A rollout policy that simulates a random playout from the given game state
/// until a terminal state is reached.
pub struct RolloutRandomly {
	/// The random number generator used to select moves during the rollout.
	rng: Box<dyn Rng>,
}

impl RolloutRandomly {
	/// Creates a new `RolloutRandomly` rollout policy with the given
	/// random number generator.
	pub fn seeded(rng: Box<dyn Rng>) -> Self {
		Self { rng }
	}

	/// Creates a new `RolloutRandomly` rollout policy with a default random
	/// number generator.
	pub fn unseeded() -> Self {
		Self {
			rng: Box::new(rand::rng()),
		}
	}
}

/// Defines how to perform a rollout (simulation) from a given game state.
pub trait RolloutPolicy {
	/// A function that simulates a playout from the given node and
	/// returns the resulting game outcome.
	fn rollout(&mut self, state: &GameState) -> GameResult;
}

impl RolloutPolicy for RolloutRandomly {
	fn rollout(&mut self, state: &GameState) -> GameResult {
		let mut simulation = state.clone();
		loop {
			if let Some(result) = simulation.determine_winner() {
				return result;
			}
			match simulation.legal_placements().choose(&mut self.rng) {
				None => return GameResult::Draw,
				Some(placement) => simulation.apply_placement(placement),
			}
		}
	}
}
