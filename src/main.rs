#![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

use std::{
	io::{self, Write},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::Duration,
};

use crate::mcts::policy::computation::Temporal;
use crate::ui::tui::display::PlacementPreview;
use crate::{
	caminos::{
		placement::LEGAL_PLACEMENTS,
		state::{GameState, Player},
	},
	mcts::agent::{MctsAgent, MctsAgentConfig},
};

mod caminos;
mod mcts;
mod ui;

fn main() {
	let arg = std::env::args().nth(1);

	match arg {
		_ => human_vs_agent(),
	};
}

fn human_vs_agent() {
	let mut state = GameState::EMPTY;

	let human = choose_human_player();

	let mut agent = MctsAgent::new(MctsAgentConfig {
		computational_limit: Box::new(Temporal {
			duration: Duration::from_secs(10),
		}),
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
		}

		if previewed {
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
