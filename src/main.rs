#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

pub mod caminos;
pub mod mcts;
pub mod util;

use std::{f64::consts::SQRT_2, time::Duration};

use crate::{
	caminos::state::{GameResult, GameState, Player},
	mcts::{
		agent::{IterativeComputationalLimit, MctsAgent, TemporalComputationalLimit},
		graph::Graph,
		policy::{
			expansion::{ExpandAlways, ExpandRandomly},
			rollout::RolloutRandomly,
			scoring::ScoringPolicy,
			selection::Ucb1,
			win::RobustChild,
		},
	},
	util::ansi,
};

fn main() {
	let mut temporal_agent = MctsAgent {
		graph: Graph::new(),
		computational_limit: Box::new(TemporalComputationalLimit {
			duration: Duration::from_millis(u64::pow(2, 10)),
		}),
		scoring_policy: ScoringPolicy {
			strong_win: 1.0,
			weak_win: 0.8,
			draw: 0.5,
			weak_loss: -1.0,
			strong_loss: -1.0,
		},
		selection_policy: Box::new(Ucb1 {
			exploration_constant: SQRT_2,
		}),
		expansion_predicate: Box::new(ExpandAlways),
		expansion_policy: Box::new(ExpandRandomly::unseeded()),
		rollout_policy: Box::new(RolloutRandomly::unseeded()),
		win_policy: Box::new(RobustChild),
	};

	let mut iterative_agent = MctsAgent {
		graph: Graph::new(),
		computational_limit: Box::new(IterativeComputationalLimit {
			iterations: u32::pow(2, 12),
		}),
		scoring_policy: ScoringPolicy {
			strong_win: 1.0,
			weak_win: 0.8,
			draw: 0.5,
			weak_loss: -1.0,
			strong_loss: -1.0,
		},
		selection_policy: Box::new(Ucb1 {
			exploration_constant: SQRT_2,
		}),
		expansion_predicate: Box::new(ExpandAlways),
		expansion_policy: Box::new(ExpandRandomly::unseeded()),
		rollout_policy: Box::new(RolloutRandomly::unseeded()),
		win_policy: Box::new(RobustChild),
	};

	let mut state = GameState::EMPTY;

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
		} else {
			println!("Game over! No valid move found");
			break;
		}
	}

	println!("{state}");
}
