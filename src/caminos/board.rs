use crate::util::ansi;

/// A Caminos bitboard representing a single layer of 8x8 cells.
/// It is player-agnostic and only tracks which cells are occupied.
///
/// It is internally represented as a [`u64`], where
/// the most significant bit maps to the top-left cell `(0, 0)` and
/// the least significant bit maps to the bottom-right cell `(7, 7)`.
///
/// The cells are indexed in rows, so in `(x, y)` notation,
/// `(0, 0)` through `(7, 0)` are the first 8 bits,
/// `(0, 1)` through `(7, 1)` are the next 8 bits, and so on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Layer {
	pub cells: u64,
}

impl Layer {
	/// A [`Layer`] with no occupied cells.
	pub const EMPTY: Self = Self::new(0x0000000000000000);

	/// A [`Layer`] with all edge cells occupied.
	pub const PERIMETER: Self = Self::new(0xFF818181818181FF);

	/// A [`Layer`] with all cells along the north edge (negative Y) occupied.
	pub const NORTH_EDGE: Self = Self::new(0xFF00000000000000);

	/// A [`Layer`] with all cells along the south edge (positive Y) occupied.
	pub const SOUTH_EDGE: Self = Self::new(0x00000000000000FF);

	/// A [`Layer`] with all cells along the east edge (positive X) occupied.
	pub const EAST_EDGE: Self = Self::new(0x8080808080808080);

	/// A [`Layer`] with all cells along the west edge (negative X) occupied.
	pub const WEST_EDGE: Self = Self::new(0x0101010101010101);

	/// Returns whether the [`Layer`] has no occupied cells.
	pub fn is_empty(&self) -> bool {
		self.cells == 0
	}

	/// Returns the bit index in a [`Layer`] for the given coordinates.
	pub fn from_xy(x: u8, y: u8) -> Self {
		if x < 8 && y < 8 {
			Self {
				cells: 1 << Self::bit_index_of_xy(x, y),
			}
		} else {
			Self::EMPTY
		}
	}

	/// Returns a [`Layer`] with an occupied cell at the given coordinates.
	pub fn bit_index_of_xy(x: u8, y: u8) -> u8 {
		63 - (y * 8 + x)
	}

	/// Returns a [`Layer`] with the given occupied cells.
	pub const fn new(cells: u64) -> Self {
		Self { cells }
	}
}

/// A Caminos bitboard consisting of 3 [`Layer`]s.
/// Like [`Layer`], it is player-agnostic and only tracks occupied cells.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BitBoard {
	pub layers: [Layer; 3],
}

impl BitBoard {
	/// A [`BitBoard`] with no occupied cells.
	pub const EMPTY: Self = Self::new([Layer::EMPTY; 3]);

	/// A [`BitBoard`] where all cells along the bottom perimeter are occupied.
	pub const BOTTOM_PERIMETER: Self = Self::new([Layer::PERIMETER, Layer::EMPTY, Layer::EMPTY]);

	/// Whether the [`BitBoard`] has no occupied cells.
	pub fn is_empty(&self) -> bool {
		self.layers[0].is_empty() && self.layers[1].is_empty() && self.layers[2].is_empty()
	}

	/// Returns a [`BitBoard`] with an occupied cell at the given coordinates.
	pub fn from_xyz(x: u8, y: u8, z: u8) -> Self {
		Self {
			layers: [
				if z == 0 { (x, y).into() } else { Layer::EMPTY },
				if z == 1 { (x, y).into() } else { Layer::EMPTY },
				if z == 2 { (x, y).into() } else { Layer::EMPTY },
			],
		}
	}

	/// Returns whether the `(x, y, z)` position is occupied.
	pub fn is_xyz_occupied(&self, x: u8, y: u8, z: u8) -> bool {
		(x < 8 && y < 8 && z < 3)
			&& (self.layers[z as usize].cells >> Layer::bit_index_of_xy(x, y) & 1 == 1)
	}

	/// Returns if there are any occupied cells that do not have an occupied
	/// cell directly below them, defying Caminos' gravity rules.
	pub fn has_floating_cells(&self) -> bool {
		let layer_2_floating = self.layers[2] & !self.layers[1];
		let layer_1_floating = self.layers[1] & !self.layers[0];
		!layer_2_floating.is_empty() || !layer_1_floating.is_empty()
	}

	pub fn has_bridge(&self, other: &BitBoard) -> bool {
		let non_covered = BitBoard {
			layers: [
				// layer 0 with no opponent above them in layer 1 or layer 2
				self.layers[0] & !other.layers[1] & !other.layers[2],
				// layer 1 with no opponent above them in layer 2
				self.layers[1] & !other.layers[2],
				// topmost layer, always non-covered
				self.layers[2],
			],
		};

		false
	}

	/// Returns a board with the given layers.
	pub const fn new(layers: [Layer; 3]) -> Self {
		Self { layers }
	}
}

impl std::fmt::Display for BitBoard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// Unoccupied: Alternate between dim ░▒ with flipped order on each row
		// Occupied: Regular █

		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let mut color = ansi::DIM;

					let char = if self.is_xyz_occupied(x, y, z) {
						color = ansi::RESET;
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{}{}{} ", color, char, ansi::RESET)?;
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

impl std::ops::Not for Layer {
	type Output = Self;

	fn not(mut self) -> Self::Output {
		self.cells = !self.cells;
		self
	}
}

impl std::ops::BitAnd for Layer {
	type Output = Self;

	fn bitand(mut self, rhs: Self) -> Self::Output {
		self.cells &= rhs.cells;
		self
	}
}

impl std::ops::BitOr for Layer {
	type Output = Self;

	fn bitor(mut self, rhs: Self) -> Self::Output {
		self.cells |= rhs.cells;
		self
	}
}

impl std::ops::BitXor for Layer {
	type Output = Self;

	fn bitxor(mut self, rhs: Self) -> Self::Output {
		self.cells ^= rhs.cells;
		self
	}
}

impl std::ops::Shl<u8> for Layer {
	type Output = Self;

	fn shl(mut self, rhs: u8) -> Self::Output {
		self.cells <<= rhs;
		self
	}
}

impl std::ops::Shr<u8> for Layer {
	type Output = Self;

	fn shr(mut self, rhs: u8) -> Self::Output {
		self.cells >>= rhs;
		self
	}
}

impl std::ops::BitAnd for BitBoard {
	type Output = Self;

	fn bitand(self, rhs: Self) -> Self::Output {
		Self {
			layers: [
				self.layers[0] & rhs.layers[0],
				self.layers[1] & rhs.layers[1],
				self.layers[2] & rhs.layers[2],
			],
		}
	}
}

impl std::ops::BitOr for BitBoard {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self {
			layers: [
				self.layers[0] | rhs.layers[0],
				self.layers[1] | rhs.layers[1],
				self.layers[2] | rhs.layers[2],
			],
		}
	}
}

impl std::ops::BitXor for BitBoard {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self::Output {
		Self {
			layers: [
				self.layers[0] ^ rhs.layers[0],
				self.layers[1] ^ rhs.layers[1],
				self.layers[2] ^ rhs.layers[2],
			],
		}
	}
}

impl From<u64> for Layer {
	fn from(cells: u64) -> Self {
		Layer { cells }
	}
}

impl From<(u8, u8)> for Layer {
	fn from(value: (u8, u8)) -> Self {
		Layer::from_xy(value.0, value.1)
	}
}

impl From<(u8, u8, u8)> for BitBoard {
	fn from(value: (u8, u8, u8)) -> Self {
		BitBoard::from_xyz(value.0, value.1, value.2)
	}
}

impl From<[u64; 3]> for BitBoard {
	fn from(value: [u64; 3]) -> Self {
		BitBoard::new([value[0].into(), value[1].into(), value[2].into()])
	}
}
