use std::sync::LazyLock;

use crate::caminos::board::BitBoard;
use crate::caminos::piece::{Piece, Rotation};

/// A single placement of a piece on the board.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Placement {
	pub piece: Piece,
	pub rotation: Rotation,
	pub position: (u8, u8, u8),
	pub board_mask: BitBoard,
}

/// Contains all legal placements for each piece type.
pub struct LegalPlacements {
	of_l: Vec<Placement>,
	of_t: Vec<Placement>,
	of_z: Vec<Placement>,
	of_o: Vec<Placement>,
}

impl LegalPlacements {
	/// Returns all possible placements in a single slice.
	pub fn all(&self) -> impl Iterator<Item = &Placement> {
		[&self.of_l, &self.of_t, &self.of_z, &self.of_o]
			.into_iter()
			.flatten()
	}

	/// Returns the slice of all possible placements for the given piece.
	pub fn of_piece(&self, piece: &Piece) -> &[Placement] {
		match piece {
			Piece::L => &self.of_l,
			Piece::T => &self.of_t,
			Piece::Z => &self.of_z,
			Piece::O => &self.of_o,
		}
	}

	/// Returns an iterator yielding all placements for the given piece that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_piece_no_overlap_no_floating(
		&self,
		piece: &Piece,
		board: &BitBoard,
	) -> impl Iterator<Item = &Placement> {
		self.of_piece(piece).iter().filter(|placement| {
			(placement.board_mask & *board).is_empty()
				&& !(placement.board_mask | *board).has_floating_cells()
		})
	}

	/// Returns an iterator yielding all placements for the given pieces that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_many_no_overlap_no_floating(
		&self,
		pieces: &[Piece],
		board: &BitBoard,
	) -> impl Iterator<Item = &Placement> {
		pieces
			.iter()
			.flat_map(|piece| self.of_piece_no_overlap_no_floating(piece, board))
	}

	/// Returns an iterator yielding all combined placements that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_all_no_overlap_no_floating(
		&self,
		board: &BitBoard,
	) -> impl Iterator<Item = &Placement> {
		self.all().filter(|placement| {
			(placement.board_mask & *board).is_empty()
				&& !(placement.board_mask | *board).has_floating_cells()
		})
	}
}

/// Precomputation of legal placements for all [`Piece`] types.
pub static LEGAL_PLACEMENTS: LazyLock<LegalPlacements> = LazyLock::new(|| LegalPlacements {
	of_l: legal_placements_of_piece(Piece::L),
	of_t: legal_placements_of_piece(Piece::T),
	of_z: legal_placements_of_piece(Piece::Z),
	of_o: legal_placements_of_piece(Piece::O),
});

/// Generates all legal placements for a given piece across the 8x8x3 board with
/// regard to all of its unique rotations.
fn legal_placements_of_piece(piece: Piece) -> Vec<Placement> {
	let mut placements = Vec::new();

	let cells = piece
		.canonical_position()
		.map(|(x, y, z)| (x as i8, y as i8, z as i8));

	for &rotation in piece.unique_rotations() {
		let rotated_cells: [(i8, i8, i8); 4] = [
			rotate_xyz(cells[0], rotation),
			rotate_xyz(cells[1], rotation),
			rotate_xyz(cells[2], rotation),
			rotate_xyz(cells[3], rotation),
		];

		// Find bounds and size of rotated piece
		let mut min_x = i8::MAX;
		let mut min_y = i8::MAX;
		let mut min_z = i8::MAX;
		let mut max_x = i8::MIN;
		let mut max_y = i8::MIN;
		let mut max_z = i8::MIN;

		for cell in &rotated_cells {
			if cell.0 < min_x {
				min_x = cell.0;
			}
			if cell.1 < min_y {
				min_y = cell.1;
			}
			if cell.2 < min_z {
				min_z = cell.2;
			}
			if cell.0 > max_x {
				max_x = cell.0;
			}
			if cell.1 > max_y {
				max_y = cell.1;
			}
			if cell.2 > max_z {
				max_z = cell.2;
			}
		}

		let width = max_x - min_x + 1;
		let height = max_y - min_y + 1;
		let depth = max_z - min_z + 1;

		// Iterate over all possible positions on the board where the piece fits
		for z_offset in 0..=(3 - depth) {
			for y_offset in 0..=(8 - height) {
				for x_offset in 0..=(8 - width) {
					let position = (x_offset as u8, y_offset as u8, z_offset as u8);

					let mut board_mask = BitBoard::EMPTY;

					for cell in &rotated_cells {
						let px = (cell.0 - min_x + x_offset) as u8;
						let py = (cell.1 - min_y + y_offset) as u8;
						let pz = (cell.2 - min_z + z_offset) as u8;
						board_mask = board_mask | BitBoard::from_xyz(px, py, pz);
					}

					let placement = Placement {
						piece,
						rotation,
						position,
						board_mask,
					};

					if !placements
						.iter()
						.any(|p: &Placement| p.board_mask == placement.board_mask)
					{
						placements.push(placement);
					}
				}
			}
		}
	}

	placements
}

/// Applies a rotation to a coordinate.
/// See https://www.euclideanspace.com/maths/algebra/matrix/orthogonal/rotation/index.htm
fn rotate_xyz(point: (i8, i8, i8), rotation: Rotation) -> (i8, i8, i8) {
	let (x, y, z) = point;

	// Map the face
	let (fx, fy, fz) = match rotation {
		Rotation::T0 | Rotation::T90 | Rotation::T180 | Rotation::T270 => (x, y, z),
		Rotation::B0 | Rotation::B90 | Rotation::B180 | Rotation::B270 => (x, -y, -z),
		Rotation::N0 | Rotation::N90 | Rotation::N180 | Rotation::N270 => (x, z, -y),
		Rotation::S0 | Rotation::S90 | Rotation::S180 | Rotation::S270 => (x, -z, y),
		Rotation::E0 | Rotation::E90 | Rotation::E180 | Rotation::E270 => (-z, y, x),
		Rotation::W0 | Rotation::W90 | Rotation::W180 | Rotation::W270 => (z, y, -x),
	};

	// Z-axis rotation
	match rotation {
		Rotation::T0 | Rotation::B0 | Rotation::N0 | Rotation::S0 | Rotation::E0 | Rotation::W0 => {
			(fx, fy, fz)
		}

		Rotation::T90
		| Rotation::B90
		| Rotation::N90
		| Rotation::S90
		| Rotation::E90
		| Rotation::W90 => (-fy, fx, fz),

		Rotation::T180
		| Rotation::B180
		| Rotation::N180
		| Rotation::S180
		| Rotation::E180
		| Rotation::W180 => (-fx, -fy, fz),

		Rotation::T270
		| Rotation::B270
		| Rotation::N270
		| Rotation::S270
		| Rotation::E270
		| Rotation::W270 => (fy, -fx, fz),
	}
}

// -------------------------------------------------------------------------- //
// UTILITY IMPLS                                                              //
// -------------------------------------------------------------------------- //

impl std::fmt::Display for Placement {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} {} {}{}{}",
			self.piece, self.rotation, self.position.0, self.position.1, self.position.2
		)
	}
}
