#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use std::io::{self, Write};

use rand::{RngExt, rng};

use crate::mcts::policy::computation::Iterative;
use crate::{
	caminos::{
		placement::LEGAL_PLACEMENTS,
		state::{GameState, Player},
	},
	mcts::agent::{MctsAgent, MctsAgentConfig},
};

pub mod caminos;
pub mod mcts;
pub mod util;

fn main() {
	let mut agent = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Iterative { iterations: 50_000 }),
		..MctsAgentConfig::default()
	});

	let mut state = GameState::EMPTY;

	let human = if rng().random_bool(0.5) {
		Player::A
	} else {
		Player::B
	};

	while state.result.is_none() {
		if state.next_player() == human {
			human_turn(&mut state);
		} else {
			agent_turn(&mut state, &mut agent);
		}

		println!("{}", state);
	}
}

fn human_turn(state: &mut GameState) {
	print!("Your turn: ");
	io::stdout().flush().unwrap();

	let mut input = String::new();
	let mut placement = None;

	while placement.is_none() {
		io::stdin().read_line(&mut input).unwrap();

		placement = LEGAL_PLACEMENTS
			.of_all_without_overlap_without_floating(state.occupancy())
			.find(|p| p.notation == input.trim());

		if placement.is_none() {
			print!("Can't place {}, try again: ", input.trim());
			io::stdout().flush().unwrap();
			input.clear();
		}
	}

	state.apply_placement(placement.unwrap());
}

fn agent_turn(state: &mut GameState, agent: &mut MctsAgent) {
	let placement = agent.search_best_placement(state.clone()).unwrap();
	state.apply_placement(placement);
	println!("Agent plays: {}", placement.notation);
}
