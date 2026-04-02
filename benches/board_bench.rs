use cabra::caminos::board::BitBoard;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_has_bridge(c: &mut Criterion) {
	let mut group = c.benchmark_group("BitBoard::has_bridge");

	let empty = BitBoard::EMPTY;
	let full = !BitBoard::EMPTY;

	let straight = BitBoard::from([
		0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
	]);

	let jagged = BitBoard::from([
		0b_11000000_01100000_00110000_00011000_00001100_00000110_00000010_00000001,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
	]);

	let snake = BitBoard::from([
		0b_00000000_01011100_01000000_01101110_00000000_01011100_01000000_01101110,
		0b_00000000_01010100_00000000_00101010_00000000_01010100_00000000_00101010,
		0b_00000011_01110110_00000000_00111010_00000010_01110110_00000000_00111011,
	]);

	let cover = BitBoard::from([
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001,
		0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
	]);

	group.bench_function("empty", |b| {
		b.iter(|| black_box(empty).has_bridge(black_box(&empty)))
	});

	group.bench_function("straight", |b| {
		b.iter(|| black_box(straight).has_bridge(black_box(&empty)))
	});

	group.bench_function("jagged", |b| {
		b.iter(|| black_box(jagged).has_bridge(black_box(&empty)))
	});

	group.bench_function("full", |b| {
		b.iter(|| black_box(full).has_bridge(black_box(&empty)))
	});

	group.bench_function("snake", |b| {
		b.iter(|| black_box(snake).has_bridge(black_box(&empty)))
	});

	group.bench_function("straight_covered", |b| {
		b.iter(|| black_box(straight).has_bridge(black_box(&cover)))
	});

	group.bench_function("jagged_covered", |b| {
		b.iter(|| black_box(jagged).has_bridge(black_box(&cover)))
	});

	group.bench_function("snake_covered", |b| {
		b.iter(|| black_box(snake).has_bridge(black_box(&cover)))
	});

	group.finish();
}

criterion_group!(benches, bench_has_bridge);
criterion_main!(benches);
