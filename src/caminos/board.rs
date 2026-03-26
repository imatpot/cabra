/// A Caminos bitboard representing 3 layers of 8x8 cells.
/// It is player-agnostic and only tracks which cells are occupied.
///
/// A layer is represented by a `u64`, where
/// the most significant bit maps to the top-left cell (0, 0) and
/// the least significant bit maps to the bottom-right cell (7, 7).
///
/// The cells of a layer are indexed in rows, so
/// (0, 0) through (7, 0) are the first 8 bits,
/// (0, 1) through (7, 1) are the next 8 bits, and so on.
///
/// Layer 0 maps to the bottom-most layer,
/// layer 1 maps to the middle layer, and
/// layer 2 maps to the top-most layer.
///
/// A coordinate like (2, 6, 1) in (x, y, z) format would, by this convention,
/// refer to the cell
/// in the 3rd column
/// on the 7th row
/// on the 2nd layer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BitBoard([u64; 3]);

impl BitBoard {
	/// Creates a new board with empty layers.
	pub const EMPTY: Self = Self([0; 3]);

	/// A board where all cells along the bottom edge are occupied.
	pub const BOTTOM_EDGE: Self = Self([0xFF818181818181FF, 0, 0]);

	/// Whether the board has no occupied cells.
	pub fn is_empty(&self) -> bool {
		self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0
	}

	/// Creates a new board with a single occupied cell
	/// at the given coordinates.
	pub fn from_xyz(x: u8, y: u8, z: u8) -> Self {
		let mut layers = [0; 3];

		if x < 8 && y < 8 && z < 3 {
			layers[z as usize] = BitBoard::layer_from_xy(x, y)
		}

		Self(layers)
	}

	/// Returns the bit index in a layer for the given (x, y) coordinates.
	pub fn index_of_xy(x: u8, y: u8) -> u8 {
		63 - (y * 8 + x)
	}

	/// Returns a bitboard layer with a single occupied cell
	/// at the given (x, y) coordinates.
	pub fn layer_from_xy(x: u8, y: u8) -> u64 {
		1 << BitBoard::index_of_xy(x, y)
	}

	/// Checks if the cell at the given (x, y, z) coordinates is occupied.
	pub fn is_xyz_occupied(&self, x: u8, y: u8, z: u8) -> bool {
		if x < 8 && y < 8 && z < 3 {
			(self.0[z as usize] >> BitBoard::index_of_xy(x, y)) & 1 == 1
		} else {
			false
		}
	}

	/// Checks if there are any occupied cells that do not have an occupied cell
	/// directly below them.
	pub fn has_floating_cells(&self) -> bool {
		for z in 1..3 {
			for y in 0..8 {
				for x in 0..8 {
					if self.is_xyz_occupied(x, y, z) && !self.is_xyz_occupied(x, y, z - 1) {
						return true;
					}
				}
			}
		}

		false
	}
}

impl std::ops::BitAnd for BitBoard {
	type Output = Self;

	fn bitand(self, rhs: Self) -> Self::Output {
		Self([
			self.0[0] & rhs.0[0],
			self.0[1] & rhs.0[1],
			self.0[2] & rhs.0[2],
		])
	}
}

impl std::ops::BitOr for BitBoard {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self([
			self.0[0] | rhs.0[0],
			self.0[1] | rhs.0[1],
			self.0[2] | rhs.0[2],
		])
	}
}

impl std::ops::BitXor for BitBoard {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self::Output {
		Self([
			self.0[0] ^ rhs.0[0],
			self.0[1] ^ rhs.0[1],
			self.0[2] ^ rhs.0[2],
		])
	}
}

impl std::fmt::Display for BitBoard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// unoccupied: alternate between ░▒ on uneven rows and ▒░ on even rows
		// occupied: █

		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let char = if self.is_xyz_occupied(x, y, z) {
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{} ", char)?;
				}

				if z < 2 {
					write!(f, " ")?;
				}
			}

			writeln!(f)?;
		}

		Ok(())
	}
}

impl From<(u8, u8, u8)> for BitBoard {
	fn from(value: (u8, u8, u8)) -> Self {
		BitBoard::from_xyz(value.0, value.1, value.2)
	}
}

impl From<&[(u8, u8, u8)]> for BitBoard {
	fn from(value: &[(u8, u8, u8)]) -> Self {
		let mut board = BitBoard::EMPTY;

		for &(x, y, z) in value {
			board = board | BitBoard::from_xyz(x, y, z);
		}

		board
	}
}
