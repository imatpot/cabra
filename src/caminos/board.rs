/// A Caminos bitboard representing 3 layers of 8x8 cells.
/// It is player-agnostic and only tracks which cells are occupied.
///
/// A layer is represented by a `u64`, where
/// the least significant bit maps to the top-left cell (0, 0) and
/// the most significant bit maps to the bottom-right cell (7, 7).
///
/// The cells of a layer are indexed in rows, so
/// (0, 0) through (7, 0) are the first 8 bits,
/// (0, 1) through (7, 1) are the next 8 bits, and so on.
///
/// Layer 0 maps to the bottom-most layer,
/// layer 1 maps to the middle layer, and
/// layer 2 maps to the top-most layer.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

	/// Returns the number of ones in the binary representation of `self`.
	pub fn count_ones(&self) -> u32 {
		self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones()
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
