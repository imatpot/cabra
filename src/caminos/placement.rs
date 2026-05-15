use std::sync::LazyLock;

use crate::caminos::board::{BitBoard, Layer};
use crate::caminos::piece::{Piece, Rotation};

/// A coordinate on the board, represented as (x, y, z).
pub type Position = (u8, u8, u8);

/// A single placement of a piece on the board.
pub struct Placement {
	/// The type of piece being placed.
	pub piece: Piece,

	/// The specific rotation of the piece in this placement
	/// which determines how the piece's cells are oriented in 3D space.
	pub rotation: Rotation,

	/// The position of the piece's reference cell.
	pub position: Position,

	/// A bitboard mask representing the cells occupied by this piece placement
	/// on the board.
	pub board_mask: BitBoard,

	/// The exact coordinates of the 4 cells occupied by this piece placement.
	pub occupied_positions: [Position; 4],

	/// A human-readable notation for this placement.
	pub notation: &'static str,

	/// Precomputed masks for quickly checking placement legality.
	pub precomputations: PlacementLegalityPrecomputation,
}

impl Placement {
	/// Checks if placing this piece on the given board would NOT result
	/// in any floating cells.
	pub fn not_floating_on(&self, board: &BitBoard) -> bool {
		(self.precomputations.layer_1_floating & !board.layers[0]).is_empty()
			&& (self.precomputations.layer_2_floating & !board.layers[1]).is_empty()
	}

	/// Checks if placing this piece on the given board would NOT overlap.
	pub fn not_overlapping_with(&self, board: &BitBoard) -> bool {
		(self.board_mask & *board).is_empty()
	}
}

/// Precomputation of floating cell masks for each layer,
/// used to quickly check if a placement introduces floating cells.
pub struct PlacementLegalityPrecomputation {
	/// A mask of all cells in layer 1 that would be floating if occupied
	/// without support from layer 0.
	pub layer_1_floating: Layer,

	/// A mask of all cells in layer 2 that would be floating if occupied
	/// without support from layer 1.
	pub layer_2_floating: Layer,
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
	pub fn all(&'static self) -> impl Iterator<Item = &'static Placement> + 'static {
		[&self.of_l, &self.of_t, &self.of_z, &self.of_o]
			.into_iter()
			.flatten()
	}

	/// Returns the slice of all possible placements for the given piece.
	pub fn of_piece(
		&'static self,
		piece: Piece,
	) -> impl Iterator<Item = &'static Placement> + 'static {
		match piece {
			Piece::L => &self.of_l,
			Piece::T => &self.of_t,
			Piece::Z => &self.of_z,
			Piece::O => &self.of_o,
		}
		.iter()
	}

	/// Returns an iterator yielding all placements for the given piece that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_piece_without_overlap_without_floating(
		&'static self,
		piece: Piece,
		board: BitBoard,
	) -> impl Iterator<Item = &'static Placement> + 'static {
		self.of_piece(piece).filter(move |placement| {
			placement.not_overlapping_with(&board) && placement.not_floating_on(&board)
		})
	}

	/// Returns an iterator yielding all placements for the given pieces that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_many_without_overlap_without_floating(
		&'static self,
		pieces: impl Iterator<Item = &'static Piece> + 'static,
		board: BitBoard,
	) -> impl Iterator<Item = &'static Placement> + 'static {
		pieces.flat_map(move |piece| self.of_piece_without_overlap_without_floating(*piece, board))
	}

	/// Returns an iterator yielding all combined placements that do
	/// not overlap with the occupied cells in the provided board and do not
	/// introduce any floating cells.
	pub fn of_all_without_overlap_without_floating(
		&'static self,
		board: BitBoard,
	) -> impl Iterator<Item = &'static Placement> + 'static {
		self.all().filter(move |placement| {
			placement.not_overlapping_with(&board) && placement.not_floating_on(&board)
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
	let mut placements = Vec::<Placement>::new();

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
					let mut cell_positions = [(0, 0, 0); 4];

					for (i, cell) in rotated_cells.iter().enumerate() {
						let px = (cell.0 - min_x + x_offset) as u8;
						let py = (cell.1 - min_y + y_offset) as u8;
						let pz = (cell.2 - min_z + z_offset) as u8;
						board_mask = board_mask | BitBoard::from_xyz(px, py, pz);
						cell_positions[i] = (px, py, pz);
					}

					if board_mask.layers[0].cells.count_ones() == 0 {
						// Piece is not touching the ground -> not legal
						continue;
					}

					let placement = Placement {
						piece,
						rotation,
						position,
						board_mask,
						occupied_positions: cell_positions,
						notation: build_notation(piece, rotation, position).leak::<'static>(),
						precomputations: PlacementLegalityPrecomputation {
							layer_1_floating: board_mask.layers[1] & !board_mask.layers[0],
							layer_2_floating: board_mask.layers[2] & !board_mask.layers[1],
						},
					};

					if !placements
						.iter()
						.any(|existing| existing.board_mask == placement.board_mask)
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

/// Builds a human-readable notation for a placement, e.g. "L T0 000".
fn build_notation(piece: Piece, rotation: Rotation, position: Position) -> String {
	format!(
		"{} {} {}{}{}",
		piece, rotation, position.0, position.1, position.2
	)
}

impl std::fmt::Display for Placement {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.notation)
	}
}

impl From<Placement> for [Position; 4] {
	fn from(placement: Placement) -> Self {
		placement.occupied_positions
	}
}
