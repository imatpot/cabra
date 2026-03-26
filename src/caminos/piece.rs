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

	/// Returns the unique rotations for this piece.
	pub fn unique_rotations(&self) -> &[Rotation] {
		match self {
			Piece::L => &[
				// No symmetries, all rotations unique
				Rotation::T0,
				Rotation::T90,
				Rotation::T180,
				Rotation::T270,
				Rotation::B0,
				Rotation::B90,
				Rotation::B180,
				Rotation::B270,
				Rotation::N0,
				Rotation::N90,
				Rotation::N180,
				Rotation::N270,
				Rotation::S0,
				Rotation::S90,
				Rotation::S180,
				Rotation::S270,
				Rotation::E0,
				Rotation::E90,
				Rotation::E180,
				Rotation::E270,
				Rotation::W0,
				Rotation::W90,
				Rotation::W180,
				Rotation::W270,
			],

			Piece::T => &[
				// Top/Bottom, North/South, and East/West are redundant
				Rotation::T0,
				Rotation::T90,
				Rotation::T180,
				Rotation::T270,
				Rotation::N0,
				Rotation::N90,
				Rotation::N180,
				Rotation::N270,
				Rotation::E0,
				Rotation::E90,
				Rotation::E180,
				Rotation::E270,
			],

			Piece::Z => &[
				// 0/180 and 90/270 degrees are redundant
				Rotation::T0,
				Rotation::T90,
				Rotation::B0,
				Rotation::B90,
				Rotation::N0,
				Rotation::N90,
				Rotation::S0,
				Rotation::S90,
				Rotation::E0,
				Rotation::E90,
				Rotation::W0,
				Rotation::W90,
			],

			Piece::O => &[
				// Top/Bottom, North/South, and East/West are redundant
				// 0/90/180/270 degrees are all redundant
				Rotation::T0,
				Rotation::N0,
				Rotation::E0,
			],
		}
	}
}

/// Orientation in 3D space as described by the 6 faces of the cube
/// (Top, Bottom, North, South, East, West) abbreviated as T, B, N, S, E, W
/// and the 4 rotations (0°, 90°, 180°, 270°).
///
/// Top is the face that is facing into positive Z,
/// Bottom is the face that is facing into negative Z,
/// North is the face that is facing into negative Y,
/// South is the face that is facing into positive Y,
/// East is the face that is facing into positive X,
/// West is the face that is facing into negative X.
#[derive(Clone, Copy)]
pub enum Rotation {
	T0,
	T90,
	T180,
	T270,
	B0,
	B90,
	B180,
	B270,
	N0,
	N90,
	N180,
	N270,
	S0,
	S90,
	S180,
	S270,
	E0,
	E90,
	E180,
	E270,
	W0,
	W90,
	W180,
	W270,
}
