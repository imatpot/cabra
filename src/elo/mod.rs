use std::{fs, path::PathBuf, time::Duration};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
	caminos::state::{GameResult, GameState, Player},
	mcts::{agent::MctsAgent, graph::Graph},
};

const K: f64 = 32.0;
const INITIAL_ELO: f64 = 1000.0;

pub struct NamedAgent {
	pub name: String,
	agent: MctsAgent,
}

impl NamedAgent {
	pub fn new(name: String, agent: MctsAgent) -> Self {
		Self { name, agent }
	}

	fn reset(&mut self) {
		self.agent.graph = Graph::new();
	}
}

#[derive(Serialize, Deserialize, Default)]
struct SearchStats {
	total_search_ms: u64,
	move_count: u64,
}

impl SearchStats {
	fn record(&mut self, duration: Duration) {
		self.total_search_ms += duration.as_millis() as u64;
		self.move_count += 1;
	}

	fn avg_search_ms(&self) -> f64 {
		if self.move_count == 0 {
			return 0.0;
		}
		self.total_search_ms as f64 / self.move_count as f64
	}
}

#[derive(Serialize, Deserialize)]
pub struct EloRecord {
	pub total_games: u32,
	pub games_per_agent: FxHashMap<String, u32>,
	pub elo: FxHashMap<String, f64>,
	pub elo_history: FxHashMap<String, Vec<f64>>,
	pub wins: FxHashMap<String, FxHashMap<String, u32>>,
	pub draws: FxHashMap<String, FxHashMap<String, u32>>,
	search_stats: FxHashMap<String, SearchStats>,
}

impl EloRecord {
	fn new(names: &[&str]) -> Self {
		let elo = names
			.iter()
			.map(|&n| (n.to_string(), INITIAL_ELO))
			.collect();
		let wins = names
			.iter()
			.map(|&n| (n.to_string(), FxHashMap::default()))
			.collect();
		let draws = names
			.iter()
			.map(|&n| (n.to_string(), FxHashMap::default()))
			.collect();
		let search_stats = names
			.iter()
			.map(|&n| (n.to_string(), SearchStats::default()))
			.collect();
		let games_per_agent = names.iter().map(|&n| (n.to_string(), 0u32)).collect();
		let elo_history = names
			.iter()
			.map(|&n| (n.to_string(), vec![INITIAL_ELO]))
			.collect();

		Self {
			total_games: 0,
			games_per_agent,
			elo,
			elo_history,
			wins,
			draws,
			search_stats,
		}
	}

	fn load_or_new(path: &PathBuf, names: &[&str]) -> Self {
		if path.exists() {
			let text = fs::read_to_string(path).expect("failed to read elo file");
			serde_json::from_str(&text).expect("failed to parse elo file")
		} else {
			Self::new(names)
		}
	}

	fn save_to_file(&self, path: &PathBuf) {
		let text = serde_json::to_string_pretty(self).expect("failed to serialize elo record");
		fs::write(path, text).expect("failed to write elo file");
	}

	fn record_search(&mut self, agent_name: &str, duration: Duration) {
		self.search_stats
			.entry(agent_name.to_string())
			.or_default()
			.record(duration);
	}

	/// https://en.wikipedia.org/wiki/Elo_rating_system#Mathematical_details
	fn update(&mut self, winner: Option<&str>, a: &str, b: &str) {
		#![allow(non_snake_case)]

		let (S_a, S_b) = match winner {
			Some(w) if w == a => (1.0, 0.0),
			Some(w) if w == b => (0.0, 1.0),
			_ => (0.5, 0.5),
		};

		let R_a = self.elo[a];
		let R_b = self.elo[b];

		let E_a = 1.0 / (1.0 + 10f64.powf((R_b - R_a) / 400.0));
		let E_b = 1.0 / (1.0 + 10f64.powf((R_a - R_b) / 400.0));

		*self.elo.get_mut(a).unwrap() = R_a + K * (S_a - E_a);
		*self.elo.get_mut(b).unwrap() = R_b + K * (S_b - E_b);

		self.total_games += 1;
		*self.games_per_agent.entry(a.to_string()).or_insert(0) += 1;
		*self.games_per_agent.entry(b.to_string()).or_insert(0) += 1;

		match winner {
			Some(w) => {
				let loser = if w == a { b } else { a };
				*self
					.wins
					.entry(w.to_string())
					.or_default()
					.entry(loser.to_string())
					.or_insert(0) += 1;
			}
			None => {
				*self
					.draws
					.entry(a.to_string())
					.or_default()
					.entry(b.to_string())
					.or_insert(0) += 1;
				*self
					.draws
					.entry(b.to_string())
					.or_default()
					.entry(a.to_string())
					.or_insert(0) += 1;
			}
		}

		self.elo_history
			.entry(a.to_string())
			.or_default()
			.push(self.elo[a]);
		self.elo_history
			.entry(b.to_string())
			.or_default()
			.push(self.elo[b]);
	}
}

fn elo_record_file_path(prefix: &str, agents: &[NamedAgent]) -> PathBuf {
	let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
	let suffix = names.join("_");
	PathBuf::from(format!("{prefix}_{suffix}.json"))
}

/// Play one game between agent_a (Player::A) and agent_b (Player::B).
/// Returns the name of the winner, or None for a draw.
fn play_game(
	agent_a: &mut NamedAgent,
	agent_b: &mut NamedAgent,
	record: &mut EloRecord,
) -> Option<String> {
	let mut state = GameState::EMPTY;

	while state.result.is_none() {
		let current = state.next_player();
		let acting = if current == Player::A {
			&mut *agent_a
		} else {
			&mut *agent_b
		};

		let result = acting.agent.search_best_placement(state.clone());
		record.record_search(&acting.name, result.duration);

		let placement = result
			.placement
			.expect("no placement found in non-terminal state");

		state.apply_placement(placement);
	}

	match state.result.unwrap() {
		GameResult::StrongWin(Player::A) | GameResult::WeakWin(Player::A) => {
			Some(agent_a.name.clone())
		}
		GameResult::StrongWin(Player::B) | GameResult::WeakWin(Player::B) => {
			Some(agent_b.name.clone())
		}
		GameResult::Draw => None,
	}
}

pub fn run_elo_tournament(mut agents: Vec<NamedAgent>, file_prefix: &str) {
	let path = elo_record_file_path(file_prefix, &agents);
	let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
	let mut record = EloRecord::load_or_new(&path, &names);

	let n = agents.len();
	let mut match_num = 0;

	loop {
		for i in 0..n {
			for j in (i + 1)..n {
				match_num += 1;
				let (left, right) = agents.split_at_mut(j);
				let agent_a = &mut left[i];
				let agent_b = &mut right[0];

				println!(
					"Match {}: {} (A) vs {} (B)",
					match_num, agent_a.name, agent_b.name
				);

				let winner = play_game(agent_a, agent_b, &mut record);
				agent_a.reset();
				agent_b.reset();

				match &winner {
					Some(w) => println!("  Winner: {w}"),
					None => println!("  Draw"),
				}

				record.update(
					winner.as_deref(),
					&agent_a.name.clone(),
					&agent_b.name.clone(),
				);
				record.save_to_file(&path);

				println!("  Standings (total games: {}):", record.total_games);
				let mut elos: Vec<_> = record.elo.iter().collect();
				elos.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
				for (name, elo) in &elos {
					let avg_ms = record
						.search_stats
						.get(*name)
						.map(|s| s.avg_search_ms())
						.unwrap_or(0.0);
					let games = record.games_per_agent.get(*name).copied().unwrap_or(0);
					println!(
						"    {}: {:.1} elo  {} games  ({:.0} ms/move avg)",
						name, elo, games, avg_ms
					);
				}
			}
		}
	}
}
