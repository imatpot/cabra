#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use rand::seq::IndexedRandom;

use crate::caminos::{board::BitBoard, piece::Piece, placement::LEGAL_PLACEMENTS};

mod caminos;

fn main() {
	println!("EMPTY\n{}", BitBoard::EMPTY);
	println!("BOTTOM EDGE\n{}", BitBoard::BOTTOM_EDGE);

	// for p in LEGAL_PLACEMENTS.of_piece(&Piece::O).iter() {
	// 	println!(
	// 		"{} ({})\n{}",
	// 		p,
	// 		if p.board_mask.has_floating_cells() {
	// 			"floating"
	// 		} else {
	// 			"legal"
	// 		},
	// 		p.board_mask,
	// 	);
	// }

	let mut rng = rand::rng();
	let random_piece = [Piece::L, Piece::T, Piece::Z, Piece::O]
		.choose(&mut rng)
		.unwrap();
	let random_placement = LEGAL_PLACEMENTS
		.of_piece(&random_piece)
		.choose(&mut rng)
		.unwrap();

	println!(
		"Random {} placement: {}\n{}",
		random_piece, random_placement, random_placement.board_mask
	);
}
