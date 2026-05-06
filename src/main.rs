#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

pub mod caminos;
pub mod mcts;
pub mod util;

use std::{
	io::{self, Write},
	time::Duration,
};

use crate::{
	caminos::{
		file::{ReadFromPath, WriteToPath},
		placement::PlacementRefs,
		state::{GameResult, GameState, Player},
	},
	mcts::{
		agent::{MctsAgent, MctsAgentConfig},
		policy::computation::{Iterative, Temporal},
	},
	util::ansi,
};

fn main() {
	let mut temporal_agent = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Temporal {
			duration: Duration::from_secs(1),
		}),
		..MctsAgentConfig::default()
	});

	let mut iterative_agent = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Iterative { iterations: 25_000 }),
		..MctsAgentConfig::default()
	});

	let mut state = GameState::EMPTY;
	let mut placements: PlacementRefs = Vec::new();

	loop {
		if let Some(result) = state.determine_winner() {
			match result {
				GameResult::StrongWin(Player::A) => {
					println!("{}Player A wins strongly!{}", ansi::GREEN, ansi::RESET)
				}
				GameResult::WeakWin(Player::A) => {
					println!("{}Player A wins weakly!{}", ansi::BLUE, ansi::RESET)
				}

				GameResult::StrongWin(Player::B) => {
					println!("{}Player B wins strongly!{}", ansi::RED, ansi::RESET)
				}
				GameResult::WeakWin(Player::B) => {
					println!("{}Player B wins weakly!{}", ansi::RED, ansi::RESET)
				}

				GameResult::Draw => println!("{}It's a draw!{}", ansi::YELLOW, ansi::RESET),
			}

			break;
		}

		let best_move = match state.next_player {
			Player::A => temporal_agent.find_best_placement(&state),
			Player::B => iterative_agent.find_best_placement(&state),
		};

		if let Some(placement) = best_move {
			println!("Player {} places {}", state.next_player, placement);
			state.apply_placement(placement);
			placements.push(placement);
		} else {
			println!("Game over! No valid move found");
			break;
		}

		println!("{state}");

		if std::env::args().any(|arg| arg == "--wait") {
			print!("Press Enter to continue...");
			io::stdout().flush().unwrap();
			let mut input = String::new();
			io::stdin().read_line(&mut input).unwrap();
			println!();
		}
	}

	let path = "result.caminos";
	placements.write_to_path(path, true).unwrap();
	let loaded_state = GameState::read_from_path(path).unwrap();
	println!("\nLoaded game state from {}:\n{}", path, loaded_state);
}
