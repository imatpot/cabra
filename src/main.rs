#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use rand::seq::IndexedRandom;

use crate::caminos::{piece::Piece, placement::LEGAL_PLACEMENTS};

mod caminos;
mod mcts;
mod util;

fn main() {
	// println!("EMPTY\n{}", BitBoard::EMPTY);
	// println!("BOTTOM EDGE\n{}", BitBoard::BOTTOM_EDGE);

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

	// let a: BitBoard = [0x7E000000000000, 0x81000000000000, 0x00000000000000].into();
	// let b: BitBoard = [0x00000000000000, 0x00000000000000, 0x00000000000000].into();

	// println!("{}\n{}\n{}", a.has_bridge(&b), a, b);
}
