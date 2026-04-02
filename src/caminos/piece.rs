/// A Caminos piece independent of its position and orientation on the board.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
	L,
	T,
	Z,
	O,
}

impl Piece {
	/// Returns the canonical position of the piece in its default
	/// orientation and location; `T0 000`.
	pub fn canonical_position(&self) -> [(u8, u8, u8); 4] {
		match self {
			// █ █ █
			// █
			Piece::L => [(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0)],

			// █ █ █
			// █
			Piece::T => [(0, 0, 0), (1, 0, 0), (2, 0, 0), (1, 1, 0)],

			// █ █
			//   █ █
			Piece::Z => [(0, 0, 0), (1, 0, 0), (1, 1, 0), (2, 1, 0)],

			// █ █
			// █ █
			Piece::O => [(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0)],
		}
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
#[derive(Clone, Copy, PartialEq, Eq)]
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

// -------------------------------------------------------------------------- //
// UTILITY IMPLS                                                              //
// -------------------------------------------------------------------------- //

impl std::fmt::Display for Piece {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Piece::L => write!(f, "L"),
			Piece::T => write!(f, "T"),
			Piece::Z => write!(f, "Z"),
			Piece::O => write!(f, "O"),
		}
	}
}

impl std::fmt::Display for Rotation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Rotation::T0 => write!(f, "T0"),
			Rotation::T90 => write!(f, "T90"),
			Rotation::T180 => write!(f, "T180"),
			Rotation::T270 => write!(f, "T270"),
			Rotation::B0 => write!(f, "B0"),
			Rotation::B90 => write!(f, "B90"),
			Rotation::B180 => write!(f, "B180"),
			Rotation::B270 => write!(f, "B270"),
			Rotation::N0 => write!(f, "N0"),
			Rotation::N90 => write!(f, "N90"),
			Rotation::N180 => write!(f, "N180"),
			Rotation::N270 => write!(f, "N270"),
			Rotation::S0 => write!(f, "S0"),
			Rotation::S90 => write!(f, "S90"),
			Rotation::S180 => write!(f, "S180"),
			Rotation::S270 => write!(f, "S270"),
			Rotation::E0 => write!(f, "E0"),
			Rotation::E90 => write!(f, "E90"),
			Rotation::E180 => write!(f, "E180"),
			Rotation::E270 => write!(f, "E270"),
			Rotation::W0 => write!(f, "W0"),
			Rotation::W90 => write!(f, "W90"),
			Rotation::W180 => write!(f, "W180"),
			Rotation::W270 => write!(f, "W27₀"),
		}
	}
}
