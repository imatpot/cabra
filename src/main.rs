#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use rand::seq::IndexedRandom;

pub mod caminos;
pub mod mcts;
pub mod util;

use crate::caminos::{board::BitBoard, piece::Piece, placement::LEGAL_PLACEMENTS};

fn main() {
	println!("EMPTY\n{}", BitBoard::EMPTY);
	println!("BOTTOM PERIMETER\n{}", BitBoard::BOTTOM_PERIMETER);

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

	for _ in 0..10 {
		let random_piece = [Piece::L, Piece::T, Piece::Z, Piece::O]
			.choose(&mut rng)
			.unwrap();

		let random_placement = LEGAL_PLACEMENTS
			.of_piece(&random_piece)
			.choose(&mut rng)
			.unwrap();

		println!("{}\n{}", random_placement, random_placement.board_mask);
	}

	let a: BitBoard = [
		0b_11000000_01111011_00001110_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
	]
	.into();

	let b: BitBoard = [
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
	]
	.into();

	println!("A (has bridge: {}):\n{}\nB:\n{}", a.has_bridge(&b), a, b);
}
