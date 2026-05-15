#![allow(non_snake_case)]

use cabra::{
	caminos::{board::BitBoard, piece::Piece, placement::LEGAL_PLACEMENTS, state::GameState},
	mcts::{
		agent::{MctsAgent, MctsAgentConfig},
		policy::{
			computation::ComputationalIntensity,
			expansion::ExpandRandomly,
			rollout::{PlacementBias, RolloutPolicy},
		},
	},
};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use pprof::{
	criterion::{Output, PProfProfiler},
	flamegraph::Options,
};
use std::hint::black_box;

criterion_main!(benches);
criterion_group!(
	name = benches;

	// TODO: all flamegraph.svg look very similar and give little insight over actual sub-calls; why?
	config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(Some(Options::default()))));

	targets =
		bench__MctsAgent__iterate,
		bench__MctsAgent__iterate_multi_rollouts,
		bench__MctsAgent__iterate_biased,
		bench__BitBoard__has_bridge,
		bench__LegalPlacements__of_piece_no_overlap_no_floating,
		bench__LegalPlacements__of_many_no_overlap_no_floating,
		bench__LegalPlacements__of_all_no_overlap_no_floating,
);

fn bench__MctsAgent__iterate(c: &mut Criterion) {
	c.bench_function("MctsAgent::iterate on empty state", |b| {
		b.iter_batched(
			|| {
				MctsAgent::new(MctsAgentConfig {
					rollout_policy: RolloutPolicy::seeded(0, &[]),
					expansion_policy: Box::new(ExpandRandomly::seeded(0)),
					computational_intensity: ComputationalIntensity {
						rollouts_per_node: 1,
						..ComputationalIntensity::default()
					},
					..MctsAgentConfig::default()
				})
			},
			|mut agent| black_box(agent.iterate(black_box(GameState::EMPTY))),
			BatchSize::SmallInput,
		)
	});
}

fn bench__MctsAgent__iterate_multi_rollouts(c: &mut Criterion) {
	c.bench_function("MctsAgent::iterate on empty state with 8 rollouts", |b| {
		b.iter_batched(
			|| {
				MctsAgent::new(MctsAgentConfig {
					rollout_policy: RolloutPolicy::seeded(0, &[]),
					expansion_policy: Box::new(ExpandRandomly::seeded(0)),
					computational_intensity: ComputationalIntensity {
						rollouts_per_node: 8,
						..ComputationalIntensity::default()
					},
					..MctsAgentConfig::default()
				})
			},
			|mut agent| black_box(agent.iterate(black_box(GameState::EMPTY))),
			BatchSize::SmallInput,
		)
	});
}

fn bench__MctsAgent__iterate_biased(c: &mut Criterion) {
	c.bench_function("MctsAgent::iterate on empty state with bias", |b| {
		b.iter_batched(
			|| {
				MctsAgent::new(MctsAgentConfig {
					rollout_policy: RolloutPolicy::seeded(
						0,
						&[
							PlacementBias::TouchingOwn(10.0),
							PlacementBias::CoverOpponent(5.0),
						],
					),
					expansion_policy: Box::new(ExpandRandomly::seeded(0)),
					..MctsAgentConfig::default()
				})
			},
			|mut agent| black_box(agent.iterate(black_box(GameState::EMPTY))),
			BatchSize::SmallInput,
		)
	});
}

fn bench__BitBoard__has_bridge(c: &mut Criterion) {
	let mut g = c.benchmark_group("BitBoard::has_bridge");

	g.bench_function("with empty board", |b| {
		b.iter(|| black_box(black_box(BitBoard::EMPTY).has_bridge(&BitBoard::EMPTY)))
	});

	g.bench_function("with full board", |b| {
		b.iter(|| black_box(black_box(!BitBoard::EMPTY).has_bridge(&BitBoard::EMPTY)))
	});

	g.bench_function("with straight bridge", |b| {
		b.iter(|| {
			black_box(
				black_box(BitBoard::from([
					0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001,
					0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
					0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
				]))
				.has_bridge(black_box(&BitBoard::EMPTY)),
			)
		})
	});

	g.bench_function("with jagged bridge", |b| {
		b.iter(|| {
			black_box(
				black_box(BitBoard::from([
					0b_11000000_01100000_00110000_00011000_00001100_00000110_00000010_00000001,
					0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
					0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
				]))
				.has_bridge(black_box(&BitBoard::EMPTY)),
			)
		})
	});

	g.bench_function("with realistic bridge", |b| {
		b.iter(|| {
			black_box(
				black_box(BitBoard::from([
					0b_11100000_00100000_00100000_00000000_00000000_00000000_00000000_00000000,
					0b_00000000_00000000_00110000_00010000_00000011_00000000_00000000_00000000,
					0b_00000000_00000000_00000000_00011110_00000000_00000000_00000000_00000000,
				]))
				.has_bridge(black_box(&BitBoard::EMPTY)),
			)
		})
	});

	g.bench_function("with snake bridge", |b| {
		b.iter(|| {
			black_box(
				black_box(BitBoard::from([
					0b_00000000_01011100_01000000_01101110_00000000_01011100_01000000_01101110,
					0b_00000000_01010100_00000000_00101010_00000000_01010100_00000000_00101010,
					0b_00000011_01110110_00000000_00111010_00000010_01110110_00000000_00111011,
				]))
				.has_bridge(black_box(&BitBoard::EMPTY)),
			)
		})
	});

	g.finish();
}

fn bench__LegalPlacements__of_all_no_overlap_no_floating(c: &mut Criterion) {
	c.bench_function("LegalPlacements::of_all_no_overlap_no_floating", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_all_without_overlap_without_floating(black_box(BitBoard::EMPTY))
					.collect::<Vec<_>>(),
			)
		})
	});
}

fn bench__LegalPlacements__of_piece_no_overlap_no_floating(c: &mut Criterion) {
	let mut g = c.benchmark_group("LegalPlacements::of_piece_no_overlap_no_floating");

	g.bench_function("with L", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_piece_without_overlap_without_floating(
						black_box(Piece::L),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with T", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_piece_without_overlap_without_floating(
						black_box(Piece::T),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with Z", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_piece_without_overlap_without_floating(
						black_box(Piece::Z),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_piece_without_overlap_without_floating(
						black_box(Piece::O),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.finish();
}

fn bench__LegalPlacements__of_many_no_overlap_no_floating(c: &mut Criterion) {
	let mut g = c.benchmark_group("LegalPlacements::of_many_no_overlap_no_floating");

	g.bench_function("with L, T", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::T].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with L, T, Z", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::T, Piece::Z].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with L, T, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::T, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with L, Z", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::Z].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with L, Z, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::Z, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with L, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::L, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with T, Z", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::T, Piece::Z].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with T, Z, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::T, Piece::Z, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with T, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::T, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.bench_function("with Z, O", |b| {
		b.iter(|| {
			black_box(
				LEGAL_PLACEMENTS
					.of_many_without_overlap_without_floating(
						black_box([Piece::Z, Piece::O].iter()),
						black_box(BitBoard::EMPTY),
					)
					.collect::<Vec<_>>(),
			)
		})
	});

	g.finish();
}
