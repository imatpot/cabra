use rand::{Rng, RngExt};

/// A Caminos piece independent of its position and orientation on the board.
#[derive(Clone, Copy)]
pub enum Piece {
	L,
	T,
	Z,
	O,
}

impl Piece {
	/// Returns a random piece.
	pub fn random(rng: &mut impl Rng) -> Self {
		match rng.random_range(0..4) {
			0 => Self::L,
			1 => Self::T,
			2 => Self::Z,
			3 => Self::O,
			_ => unreachable!(),
		}
	}

	/// Returns a random piece from the given slice of pieces.
	pub fn random_of(rng: &mut impl Rng, pieces: &[Piece]) -> Option<Self> {
		if pieces.is_empty() {
			return None;
		}

		let i = rng.random_range(0..pieces.len());
		Some(pieces[i])
	}
}
