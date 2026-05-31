#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use std::{
	f32::consts::SQRT_2,
	io::{self, Write},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::Duration,
};

use crate::{
	caminos::piece::Piece,
	mcts::policy::{
		computation::Iterative,
		rollout::{PlacementBias, RolloutPolicy},
		selection::BayesianUct,
	},
};
use crate::{
	caminos::{
		placement::LEGAL_PLACEMENTS,
		state::{GameState, Player},
	},
	mcts::agent::{MctsAgent, MctsAgentConfig},
};
use crate::{mcts::graph::Graph, ui::tui::display::PlacementPreview};

mod caminos;
mod mcts;
mod ui;

fn main() {
	let arg = std::env::args().nth(1);

	match arg.as_deref() {
		Some("count") => count(),
		Some("avg") => avg_moves(),
		_ => human_vs_agent(),
	};
}

fn human_vs_agent() {
	let mut state = GameState::EMPTY;

	let human = choose_human_player();

	let mut agent = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Iterative {
			iterations: 200_000,
		}),
		selection_policy: Box::new(BayesianUct {
			exploration_constant: SQRT_2,
			alpha: 1.0,
			beta: 1.0,
		}),
		rollout_policy: RolloutPolicy::unseeded(&[
			PlacementBias::CoverOpponent(2.0),
			PlacementBias::TouchingOwn(2.0),
			PlacementBias::EastWest(2.0),
			PlacementBias::NorthSouth(0.8),
			PlacementBias::Piece(Piece::L, 0.8),
		]),
		..MctsAgentConfig::default()
	});

	while state.result.is_none() {
		if state.next_player() == human {
			human_turn(&mut state);
		} else {
			agent_turn(&mut state, &mut agent);
		}
	}
}

fn choose_human_player() -> Player {
	loop {
		print!("Choose your turn (1 = Player A, 2 = Player B): ");
		io::stdout().flush().unwrap();

		let mut input = String::new();
		io::stdin().read_line(&mut input).unwrap();

		match input.trim() {
			"1" => return Player::A,
			"2" => return Player::B,
			_ => println!("Please enter 1 or 2."),
		}
	}
}

fn human_turn(state: &mut GameState) {
	print!("Your turn: ");
	io::stdout().flush().unwrap();

	let mut input = String::new();
	let mut yes_no = String::new();
	let mut placement = None;

	let mut previewed = false;

	while placement.is_none() {
		io::stdin().read_line(&mut input).unwrap();

		if input.trim().ends_with("?") {
			previewed = true;
			input = input.trim().trim_end_matches("?").to_string();
		}

		placement = LEGAL_PLACEMENTS
			.of_all_without_overlap_without_floating(state.occupancy())
			.find(|p| p.notation == input.trim());

		if placement.is_none() {
			print!("Can't place {}, try again: ", input.trim());
			io::stdout().flush().unwrap();
			input.clear();
		} else if previewed {
			println!("{}", PlacementPreview(state, placement));

			print!(
				"Do you want to place {}? (y/n): ",
				placement.unwrap().notation
			);
			io::stdout().flush().unwrap();

			io::stdin().read_line(&mut yes_no).unwrap();
			if yes_no.trim().eq_ignore_ascii_case("y") {
				break;
			} else {
				placement = None;
				yes_no.clear();
				input.clear();
				print!("Your turn: ");
				io::stdout().flush().unwrap();
			}
		}
	}

	state.apply_placement(placement.unwrap());

	if !previewed {
		println!("{}", PlacementPreview(&state, placement));
	}
}

fn agent_turn(state: &mut GameState, agent: &mut MctsAgent) {
	let running = Arc::new(AtomicBool::new(true));
	let spinner_running = Arc::clone(&running);

	let spinner_thread = thread::spawn(move || {
		let frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
		let mut i = 0;

		while spinner_running.load(Ordering::Relaxed) {
			print!("\rAgent thinking {} ", frames[i]);
			io::stdout().flush().unwrap();
			i = (i + 1) % frames.len();
			thread::sleep(Duration::from_millis(80));
		}

		print!("\r{}\r", " ".repeat(24));
		io::stdout().flush().unwrap();
	});

	let placement = agent
		.search_best_placement(state.clone())
		.placement
		.unwrap();

	running.store(false, Ordering::Relaxed);
	let _ = spinner_thread.join();

	state.apply_placement(placement);
	println!("Agent plays: {}", placement.notation);
	println!("{}", PlacementPreview(&state, Some(placement)));
}

fn count() {
	let mut counts = [0usize; 5];
	count_reachable(&GameState::EMPTY, 0, 4, &mut counts);
	for (depth, count) in counts.iter().enumerate() {
		println!("Layer {}: {} states", depth, count);
	}
}

fn count_reachable(state: &GameState, depth: usize, max_depth: usize, counts: &mut [usize]) {
	counts[depth] += 1;
	if depth == max_depth || state.result.is_some() {
		return;
	}
	for placement in LEGAL_PLACEMENTS.of_all_without_overlap_without_floating(state.occupancy()) {
		let mut next = state.clone();
		next.apply_placement(placement);
		count_reachable(&next, depth + 1, max_depth, counts);
	}
}

fn avg_moves() {
	let mut a = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Iterative { iterations: 1 }),
		..MctsAgentConfig::default()
	});

	let mut b = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Iterative { iterations: 1 }),
		..MctsAgentConfig::default()
	});

	let mut game = GameState::EMPTY;
	let mut legal_move_total = 0usize;
	let mut move_total = 0usize;
	let mut num_games = 0usize;

	let mut legal_moves_on_turn = [0usize; 28];
	let mut legal_moves_on_turn_min = [0usize; 28];
	let mut legal_moves_on_turn_max = [0usize; 28];
	let mut num_occurences_turn = [0usize; 28];

	loop {
		let mut turn = 0usize;

		while game.result.is_none() {
			let legal_move_count = game.next_legal_placements().count();
			legal_move_total += legal_move_count;

			num_occurences_turn[turn] += 1;
			legal_moves_on_turn[turn] += legal_move_count;

			if legal_move_count < legal_moves_on_turn_min[turn]
				|| legal_moves_on_turn_min[turn] == 0
			{
				legal_moves_on_turn_min[turn] = legal_move_count;
			}

			if legal_move_count > legal_moves_on_turn_max[turn] {
				legal_moves_on_turn_max[turn] = legal_move_count;
			}

			let playing_agent = if game.next_player() == Player::A {
				&mut a
			} else {
				&mut b
			};

			let placement = playing_agent
				.search_best_placement(game.clone())
				.placement
				.unwrap();

			game.apply_placement(placement);
			move_total += 1;
			turn += 1;
		}

		num_games += 1;

		println!(
			"Stats after Game {} ({})\n  Total moves: {}\n  Avg. amount of legal moves per turn: {}\n  Avg. number of moves per turn: {:?}\n  Min number of moves per turn: {:?}\n  Max number of moves per turn: {:?}",
			num_games,
			game.result.unwrap(),
			move_total,
			legal_move_total as f64 / move_total as f64,
			legal_moves_on_turn
				.iter()
				.zip(num_occurences_turn.iter())
				.map(|(lm, n)| if *n > 0 { *lm as f64 / *n as f64 } else { 0.0 })
				.collect::<Vec<_>>(),
			legal_moves_on_turn_min,
			legal_moves_on_turn_max,
		);

		// Reset agents
		a.graph = Graph::new();
		b.graph = Graph::new();

		// Reset game
		game = GameState::EMPTY;
	}
}
