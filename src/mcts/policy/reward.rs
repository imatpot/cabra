use crate::caminos::state::{GameResult, Player};

/// Defines how to assign scores to game results for a specific player.
pub struct RewardPolicy {
	/// The score assigned to a strong win for the player.
	pub strong_win: f32,

	/// The score assigned to a weak win for the player.
	pub weak_win: f32,

	/// The score assigned to a draw.
	pub draw: f32,

	/// The score assigned to a weak loss for the player.
	pub weak_loss: f32,

	/// The score assigned to a strong loss for the player.
	pub strong_loss: f32,
}

impl Default for RewardPolicy {
	fn default() -> Self {
		Self {
			strong_win: 1.0,
			weak_win: 0.8,
			draw: 0.5,
			weak_loss: -1.0,
			strong_loss: -1.0,
		}
	}
}

impl RewardPolicy {
	/// Returns the score corresponding to the given game result
	/// from the perspective of the given player.
	pub fn score(&self, result: &GameResult, player: Player) -> f32 {
		match result {
			GameResult::StrongWin(p) if *p == player => self.strong_win,
			GameResult::WeakWin(p) if *p == player => self.weak_win,
			GameResult::Draw => self.draw,
			GameResult::WeakWin(_) => self.weak_loss,
			GameResult::StrongWin(_) => self.strong_loss,
		}
	}
}
