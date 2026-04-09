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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
	pub cells: u64,
}

impl Layer {
	/// A [`Layer`] with no occupied cells.
	pub const EMPTY: Self =
		Self::new(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

	/// A [`Layer`] with all edge cells occupied.
	pub const PERIMETER: Self =
		Self::new(0b_11111111_10000001_10000001_10000001_10000001_10000001_10000001_11111111);

	/// A [`Layer`] with all cells along the north edge (negative Y) occupied.
	pub const NORTH: Self =
		Self::new(0b_11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000);

	/// A [`Layer`] with all cells along the east edge (positive X) occupied.
	pub const EAST: Self =
		Self::new(0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001);

	/// A [`Layer`] with all cells along the south edge (positive Y) occupied.
	pub const SOUTH: Self =
		Self::new(0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111);

	/// A [`Layer`] with all cells along the west edge (negative X) occupied.
	pub const WEST: Self =
		Self::new(0b_10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000);

	/// Shifts all cells shifted one step to the north (negative Y).
	/// Does not wrap around.
	pub fn shift_north(self) -> Self {
		(self & !Self::NORTH) << 8
	}

	/// Shifts all cells shifted one step to the east (positive X).
	/// Does not wrap around.
	pub fn shift_east(self) -> Self {
		(self & !Self::EAST) >> 1
	}

	/// Shifts all cells shifted one step to the south (positive Y).
	/// Does not wrap around.
	pub fn shift_south(self) -> Self {
		(self & !Self::SOUTH) >> 8
	}

	/// Shifts all cells shifted one step to the west (negative X).
	/// Does not wrap around.
	pub fn shift_west(self) -> Self {
		(self & !Self::WEST) << 1
	}

	/// Shifts all cells one step in all four cardinal directions.
	/// Does not wrap around.
	pub fn shift_cardinally(self) -> Self {
		self.shift_north() | self.shift_east() | self.shift_south() | self.shift_west()
	}

	/// Returns a [`Layer`] with an occupied cell at the given coordinates.
	pub fn bit_index_of_xy(x: u8, y: u8) -> u8 {
		63 - (y * 8 + x)
	}

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

	/// Returns a [`Layer`] with the given occupied cells.
	pub const fn new(cells: u64) -> Self {
		Self { cells }
	}
}

/// A Caminos bitboard consisting of 3 [`Layer`]s.
/// Like [`Layer`], it is player-agnostic and only tracks occupied cells.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BitBoard {
	pub layers: [Layer; 3],
}

impl BitBoard {
	/// A [`BitBoard`] with no occupied cells.
	pub const EMPTY: Self = Self::new([Layer::EMPTY; 3]);

	/// A [`BitBoard`] where all cells along the bottom perimeter are occupied.
	pub const BOTTOM_PERIMETER: Self = Self::new([Layer::PERIMETER, Layer::EMPTY, Layer::EMPTY]);

	/// A [`BitBoard`] where all cells in the north (negative Y) are occupied.
	pub const NORTH: Self = Self::new([Layer::NORTH; 3]);

	/// A [`BitBoard`] where all cells in the east (positive X) are occupied.
	pub const EAST: Self = Self::new([Layer::EAST; 3]);

	/// A [`BitBoard`] where all cells in the south (positive Y) are occupied.
	pub const SOUTH: Self = Self::new([Layer::SOUTH; 3]);

	/// A [`BitBoard`] where all cells in the west (negative X) are occupied.
	pub const WEST: Self = Self::new([Layer::WEST; 3]);

	/// Shifts all cells shifted one step to the north (negative Y).
	/// Does not wrap around.
	pub fn shift_north(self) -> Self {
		(self & !Self::NORTH) << 8
	}

	/// Shifts all cells shifted one step to the east (positive X).
	/// Does not wrap around.
	pub fn shift_east(self) -> Self {
		(self & !Self::EAST) >> 1
	}

	/// Shifts all cells shifted one step to the south (positive Y).
	/// Does not wrap around.
	pub fn shift_south(self) -> Self {
		(self & !Self::SOUTH) >> 8
	}

	/// Shifts all cells shifted one step to the west (negative X).
	/// Does not wrap around.
	pub fn shift_west(self) -> Self {
		(self & !Self::WEST) << 1
	}

	/// Shifts all cells one step in all four cardinal directions.
	/// Does not wrap around.
	pub fn shift_cardinally(self) -> Self {
		self.shift_north() | self.shift_east() | self.shift_south() | self.shift_west()
	}

	/// Whether the [`BitBoard`] has no occupied cells.
	pub fn is_empty(&self) -> bool {
		self.layers[0].is_empty() && self.layers[1].is_empty() && self.layers[2].is_empty()
	}

	/// Returns if there are any occupied cells that do not have an occupied
	/// cell directly below them, defying Caminos' gravity rules.
	pub fn has_floating_cells(&self) -> bool {
		let layer_2_floating = self.layers[2] & !self.layers[1];
		let layer_1_floating = self.layers[1] & !self.layers[0];
		!layer_2_floating.is_empty() || !layer_1_floating.is_empty()
	}

	/// Returns whether the `(x, y, z)` position is occupied.
	pub fn is_xyz_occupied(&self, x: u8, y: u8, z: u8) -> bool {
		(x < 8 && y < 8 && z < 3)
			&& (self.layers[z as usize].cells >> Layer::bit_index_of_xy(x, y) & 1 == 1)
	}

	/// Returns whether there is a valid bridge spanning from north to south
	/// or west to east, according to Caminos' bridging rules.
	pub fn has_bridge(&self, other: &BitBoard) -> bool {
		let usable: BitBoard = [
			// Layer 0 with no opponent above them in layer 1 or layer 2
			self.layers[0] & !other.layers[1] & !other.layers[2],
			// Layer 1 with no opponent above them in layer 2
			self.layers[1] & !other.layers[2],
			// Topmost layer, always non-covered
			self.layers[2],
		]
		.into();

		// Begin north and west
		let mut visited_n_s = usable & Self::NORTH;
		let mut visited_w_e = usable & Self::WEST;
		let mut next_n_s = visited_n_s;
		let mut next_w_e = visited_w_e;

		/// Expands the given bitboard by one step in all manners which are
		/// allowed by Caminos' bridging rules. Specifically,
		///
		/// - Cardinally on the same layer
		/// - Cardinally up or down one layer
		/// - Straight up or down one layer
		fn expand(base: &BitBoard) -> BitBoard {
			let cardinals = base.shift_cardinally();
			return [
				(
					// Cardinally on same layer
					cardinals.layers[0]
    				// Cardinally down from layer 1
    				| cardinals.layers[1]
    				// Straight down from layer 1
    				| base.layers[1]
				),
				(
					// Cardinally on same layer
					cardinals.layers[1]
                    // Cardinally up from layer 0
                    | cardinals.layers[0]
                    // Cardinally shifted down from layer 1
                    | cardinals.layers[2]
                    // Straight up from layer 0
                    | base.layers[0]
                    // Straight down from layer 2
                    | base.layers[2]
				),
				(
					// Cardinally on same layer
					cardinals.layers[2]
					// Cardinally up from layer 1
					| cardinals.layers[1]
					// Straight up from layer 1
					| base.layers[1]
				),
			]
			.into();
		}

		// Expand north-south and west-east at the same time, until no more
		// expansions are happening or a bridge is found
		loop {
			if !next_n_s.is_empty() {
				// Expand into unvisited usable cells if there was an expansion
				next_n_s = usable & !visited_n_s & expand(&next_n_s);
			}

			if !next_w_e.is_empty() {
				// Expand into unvisited usable cells if there was an expansion
				next_w_e = usable & !visited_w_e & expand(&next_w_e);
			}

			if !(next_n_s & Self::SOUTH).is_empty() || !(next_w_e & Self::EAST).is_empty() {
				// South or east reached
				return true;
			}

			if next_n_s.is_empty() && next_w_e.is_empty() {
				// No more expansions
				return false;
			}

			// Extend visited cells by expansion
			visited_n_s = visited_n_s | next_n_s;
			visited_w_e = visited_w_e | next_w_e;
		}
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

	/// Returns a board with the given layers.
	pub const fn new(layers: [Layer; 3]) -> Self {
		Self { layers }
	}
}

// -------------------------------------------------------------------------- //
// UTILITY IMPLS                                                              //
// -------------------------------------------------------------------------- //

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

impl std::ops::Not for BitBoard {
	type Output = Self;

	fn not(mut self) -> Self::Output {
		self.layers[0].cells = !self.layers[0].cells;
		self.layers[1].cells = !self.layers[1].cells;
		self.layers[2].cells = !self.layers[2].cells;
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

impl std::ops::Shl<u8> for BitBoard {
	type Output = Self;

	fn shl(self, rhs: u8) -> Self::Output {
		Self {
			layers: [
				self.layers[0] << rhs,
				self.layers[1] << rhs,
				self.layers[2] << rhs,
			],
		}
	}
}

impl std::ops::Shr<u8> for BitBoard {
	type Output = Self;

	fn shr(self, rhs: u8) -> Self::Output {
		Self {
			layers: [
				self.layers[0] >> rhs,
				self.layers[1] >> rhs,
				self.layers[2] >> rhs,
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
		BitBoard::new([
			Layer::new(value[0]),
			Layer::new(value[1]),
			Layer::new(value[2]),
		])
	}
}

impl From<[Layer; 3]> for BitBoard {
	fn from(layers: [Layer; 3]) -> Self {
		BitBoard::new(layers)
	}
}

// -------------------------------------------------------------------------- //
// TESTS                                                                      //
// -------------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn layer_shift_north() {
		let layer = Layer::from((3, 4));
		let shifted = layer.shift_north();

		assert_eq!(shifted, Layer::from((3, 3)));
	}

	#[test]
	fn layer_shift_north_no_wrap() {
		let layer = Layer::from((3, 0));
		let shifted = layer.shift_north();

		assert_eq!(shifted, Layer::EMPTY);
	}

	#[test]
	fn layer_shift_east() {
		let layer = Layer::from((3, 4));
		let shifted = layer.shift_east();

		assert_eq!(shifted, Layer::from((4, 4)));
	}

	#[test]
	fn layer_shift_east_no_wrap() {
		let layer = Layer::from((7, 4));
		let shifted = layer.shift_east();

		assert_eq!(shifted, Layer::EMPTY);
	}

	#[test]
	fn layer_shift_south() {
		let layer = Layer::from((3, 4));
		let shifted = layer.shift_south();

		assert_eq!(shifted, Layer::from((3, 5)));
	}

	#[test]
	fn layer_shift_south_no_wrap() {
		let layer = Layer::from((3, 7));
		let shifted = layer.shift_south();

		assert_eq!(shifted, Layer::EMPTY);
	}

	#[test]
	fn layer_shift_west() {
		let layer = Layer::from((3, 4));
		let shifted = layer.shift_west();

		assert_eq!(shifted, Layer::from((2, 4)));
	}

	#[test]
	fn layer_shift_west_no_wrap() {
		let layer = Layer::from((0, 4));
		let shifted = layer.shift_west();

		assert_eq!(shifted, Layer::EMPTY);
	}

	#[test]
	fn layer_shift_cardinally() {
		let layer = Layer::from((3, 3));
		let shifted = layer.shift_cardinally();

		assert_eq!(
			shifted,
			Layer::from((3, 2)) | Layer::from((4, 3)) | Layer::from((3, 4)) | Layer::from((2, 3))
		);
	}

	#[test]
	fn bitboard_shift_north() {
		let board = BitBoard::from((1, 1, 1)) & BitBoard::from((2, 2, 2));
		let shifted = board.shift_north();

		assert_eq!(
			shifted,
			BitBoard::from((1, 0, 1)) & BitBoard::from((2, 1, 2))
		);
	}

	#[test]
	fn bitboard_shift_east() {
		let board = BitBoard::from((1, 1, 1)) & BitBoard::from((2, 2, 2));
		let shifted = board.shift_east();

		assert_eq!(
			shifted,
			BitBoard::from((2, 1, 1)) & BitBoard::from((3, 2, 2))
		);
	}

	#[test]
	fn bitboard_shift_south() {
		let board = BitBoard::from((1, 1, 1)) & BitBoard::from((2, 2, 2));
		let shifted = board.shift_south();

		assert_eq!(
			shifted,
			BitBoard::from((1, 2, 1)) & BitBoard::from((2, 3, 2))
		);
	}

	#[test]
	fn bitboard_shift_west() {
		let board = BitBoard::from((1, 1, 1)) & BitBoard::from((2, 2, 2));
		let shifted = board.shift_west();

		assert_eq!(
			shifted,
			BitBoard::from((0, 1, 1)) & BitBoard::from((1, 2, 2))
		);
	}

	#[test]
	fn bitboard_has_floating_cells_false() {
		let board = BitBoard::from((1, 1, 0));

		assert!(!board.has_floating_cells());
	}

	#[test]
	fn bitboard_has_floating_cells_true() {
		let board = BitBoard::from((1, 1, 1));

		assert!(board.has_floating_cells());
	}

	#[test]
	fn bitboard_has_bridge_false_empty() {
		let board = BitBoard::from([
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(!board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_false_1_gap() {
		let board = BitBoard::from([
			0b_00001000_00001000_00001000_00000000_00001000_00001000_00001000_00001000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(!board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_false_diagonal() {
		let board = BitBoard::from([
			0b_10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(!board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_false_covered() {
		let board = BitBoard::from([
			0b_10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		let other = BitBoard::from([
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(!board.has_bridge(&other));
	}

	#[test]
	fn bitboard_has_bridge_true_straight() {
		let board = BitBoard::from([
			0b_00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_true_same_layer_jagged() {
		let board = BitBoard::from([
			0b_00110000_00011000_00001100_00000110_00000011_00000001_00000001_00000001,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_true_staircase() {
		let board = BitBoard::from([
			0b_10000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_01010101_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_00101000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(board.has_bridge(&BitBoard::EMPTY));
	}

	#[test]
	fn bitboard_has_bridge_true_straight_down() {
		let board = BitBoard::from([
			0b_01111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		]);

		assert!(board.has_bridge(&BitBoard::EMPTY));
	}
}
