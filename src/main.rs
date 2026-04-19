use std::time::Duration;

use macroquad::miniquad;
use macroquad::prelude::*;

fn conf() -> Conf {
	Conf {
		window_title: "3D Cubes".to_string(),
		window_width: 960,
		window_height: 720,
		sample_count: 4,
		high_dpi: true,
		platform: miniquad::conf::Platform {
			linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
			// swap_interval: Some(60),
			..Default::default()
		},
		..Default::default()
	}
}

fn world_to_screen(point: Vec3, cam: &Camera3D) -> Option<Vec2> {
	let view = Mat4::look_at_rh(cam.position, cam.target, cam.up);
	let aspect = screen_width() / screen_height();
	let proj = Mat4::perspective_rh_gl(cam.fovy, aspect, 0.01, 1000.0);
	let clip = proj * view * point.extend(1.0);
	if clip.w <= 0.0 {
		return None;
	}
	let ndc = clip.xyz() / clip.w;
	if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 {
		return None;
	}
	Some(vec2(
		(ndc.x + 1.0) * 0.5 * screen_width(),
		(1.0 - ndc.y) * 0.5 * screen_height(),
	))
}

fn draw_label(text: &str, world_pos: Vec3, cam: &Camera3D, color: Color, font_size: f32) {
	if let Some(s) = world_to_screen(world_pos, cam) {
		let w = measure_text(text, None, font_size as u16, 1.0).width;
		draw_text(
			text,
			s.x - w * 0.5,
			s.y + font_size * 0.35,
			font_size,
			color,
		);
	}
}

fn button(label: &str, x: f32, y: f32, font_size: f32) -> bool {
	button_colored(label, x, y, font_size, Color::from_rgba(40, 40, 55, 200))
}

fn button_colored(label: &str, x: f32, y: f32, font_size: f32, base_bg: Color) -> bool {
	let pad_x = 10.0;
	let pad_y = 5.0;
	let m = measure_text(label, None, font_size as u16, 1.0);
	let w = m.width + pad_x * 2.0;
	let h = font_size + pad_y * 2.0;

	let mouse = Vec2::from(mouse_position());
	let hovered = mouse.x >= x && mouse.x <= x + w && mouse.y >= y && mouse.y <= y + h;
	let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

	let bg = if clicked {
		Color::from_rgba(200, 200, 200, 220)
	} else if hovered {
		Color::from_rgba(
			(base_bg.r * 255.0 + 40.0).min(255.0) as u8,
			(base_bg.g * 255.0 + 40.0).min(255.0) as u8,
			(base_bg.b * 255.0 + 40.0).min(255.0) as u8,
			220,
		)
	} else {
		base_bg
	};
	let fg = if clicked { BLACK } else { WHITE };
	let border = if hovered {
		Color::from_rgba(180, 180, 220, 255)
	} else {
		Color::from_rgba(90, 90, 110, 255)
	};

	draw_rectangle(x, y, w, h, bg);
	draw_rectangle_lines(x, y, w, h, 1.5, border);
	draw_text(label, x + pad_x, y + pad_y + font_size * 0.8, font_size, fg);
	clicked
}

pub fn draw_cube_wires_thick(position: Vec3, size: Vec3, color: Color, thickness: f32) {
	let hx = size.x * 0.5;
	let hy = size.y * 0.5;
	let hz = size.z * 0.5;
	let center = position;
	let xs = [-hx, hx];
	let ys = [-hy, hy];
	let zs = [-hz, hz];
	for &y in &ys {
		for &z in &zs {
			draw_cube(
				center + vec3(0.0, y, z),
				vec3(size.x, thickness, thickness),
				None,
				color,
			);
		}
	}
	for &x in &xs {
		for &z in &zs {
			draw_cube(
				center + vec3(x, 0.0, z),
				vec3(thickness, size.y, thickness),
				None,
				color,
			);
		}
	}
	for &x in &xs {
		for &y in &ys {
			draw_cube(
				center + vec3(x, y, 0.0),
				vec3(thickness, thickness, size.z),
				None,
				color,
			);
		}
	}
}

fn draw_fps(fps: i32) {
	let fps_color = if fps >= 55 {
		Color::from_rgba(100, 220, 100, 255)
	} else if fps >= 30 {
		Color::from_rgba(220, 200, 80, 255)
	} else {
		Color::from_rgba(220, 80, 80, 255)
	};
	draw_text(&format!("{:03} fps", fps), 14.0, 60.0, 18.0, fps_color);
}

fn draw_pause_quit_buttons(paused: bool, btn_y: f32, btn_size: f32) -> (bool, bool) {
	let pause_label = if paused {
		"Resume [Space]"
	} else {
		"Pause [Space]"
	};
	let pause_bg = if paused {
		Color::from_rgba(140, 40, 40, 200)
	} else {
		Color::from_rgba(40, 120, 40, 200)
	};
	let pause_clicked = button_colored(pause_label, 14.0, btn_y, btn_size, pause_bg);
	let quit_x = 14.0 + measure_text(pause_label, None, btn_size as u16, 1.0).width + 40.0;
	let quit_clicked = button("Quit [Q]", quit_x, btn_y, btn_size);
	(pause_clicked, quit_clicked)
}

#[macroquad::main(conf)]
async fn main() {
	let mut cubes: Vec<Vec<Vec<Option<bool>>>> = (0..8)
		.map(|x| {
			(0..8)
				.map(|y| (0..3).map(|z| Some((x + y + z) % 2 == 0)).collect())
				.collect()
		})
		.collect();

	let cube_size = 1.0f32;
	let gap = 0.015f32;
	let step = cube_size + gap;
	let margin = step * 1.4;

	let mut yaw: f32 = 45.0f32.to_radians();
	let mut pitch: f32 = -35.0f32.to_radians();
	let mut camera_dist = 22.0f32;

	let color_a = Color::from_rgba(70, 130, 220, 255);
	let color_b = Color::from_rgba(220, 90, 70, 255);
	let lbl_color = Color::from_rgba(240, 230, 150, 255);
	let lbl_x = Color::from_rgba(255, 110, 110, 255);
	let lbl_y = Color::from_rgba(110, 220, 110, 255);
	let lbl_z = Color::from_rgba(110, 160, 255, 255);
	let lsize = 15.0f32;
	let asize = 17.0f32;

	let mut last_mouse = Vec2::ZERO;
	let mut dragging = false;
	let mut drag_moved = false;
	let mut paused = false;

	let fps_update_interval = Duration::from_millis(200);
	let mut now = std::time::Instant::now() - fps_update_interval;
	let mut fps = 0;

	// We keep one cached camera so the paused screen can still project labels
	// (not needed when paused, but keeps the type available).
	#[allow(unused_assignments)]
	let mut camera = Camera3D {
		position: vec3(0.0, 10.0, 20.0),
		target: vec3(0.0, 0.0, 0.0),
		up: vec3(0.0, 1.0, 0.0),
		..Default::default()
	};

	let btn_size = 18.0f32;

	loop {
		// ── Always: toggle pause / quit ────────────────────────────────────────
		if is_key_pressed(KeyCode::Space) {
			paused = !paused;
		}
		if is_key_pressed(KeyCode::Q) {
			break;
		}

		// ── PAUSED: minimal render — black bg, fps, pause+quit buttons only ────
		if paused {
			clear_background(BLACK);
			if now.elapsed() >= fps_update_interval {
				fps = get_fps();
				now = std::time::Instant::now();
			}
			draw_fps(fps);
			let btn_y = screen_height() - 44.0;
			let (pause_clicked, quit_clicked) = draw_pause_quit_buttons(paused, btn_y, btn_size);
			if pause_clicked {
				paused = false;
			}
			if quit_clicked {
				break;
			}
			next_frame().await;
			continue;
		}

		// ── RUNNING: full input ────────────────────────────────────────────────
		let rot_speed = 0.025f32;
		let zoom_speed = 0.4f32;
		let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

		if is_key_down(KeyCode::Left) {
			yaw -= rot_speed;
		}
		if is_key_down(KeyCode::Right) {
			yaw += rot_speed;
		}
		if !shift {
			if is_key_down(KeyCode::Up) {
				pitch = (pitch + rot_speed).min(1.4);
			}
			if is_key_down(KeyCode::Down) {
				pitch = (pitch - rot_speed).max(-1.4);
			}
		} else {
			if is_key_down(KeyCode::Up) {
				camera_dist = (camera_dist - zoom_speed).max(4.0);
			}
			if is_key_down(KeyCode::Down) {
				camera_dist = (camera_dist + zoom_speed).min(60.0);
			}
		}

		let scroll = mouse_wheel().1;
		if scroll != 0.0 {
			camera_dist = (camera_dist - scroll * 1.2).clamp(4.0, 60.0);
		}

		if is_mouse_button_pressed(MouseButton::Left) {
			dragging = true;
			drag_moved = false;
			last_mouse = Vec2::from(mouse_position());
		}
		if is_mouse_button_released(MouseButton::Left) {
			dragging = false;
		}
		if dragging {
			let mouse = Vec2::from(mouse_position());
			let delta = mouse - last_mouse;
			if delta.length() > 0.5 {
				drag_moved = true;
				yaw += delta.x * 0.005;
				pitch = (pitch + delta.y * 0.005).clamp(-1.4, 1.4);
			}
			last_mouse = mouse;
		}

		let mut delete_one = is_key_pressed(KeyCode::X);

		// ── Camera ─────────────────────────────────────────────────────────────
		let grid_center = vec3(3.5 * step, 1.0 * step, 3.5 * step);
		let cam_offset = vec3(
			camera_dist * yaw.cos() * pitch.cos(),
			camera_dist * pitch.sin(),
			camera_dist * yaw.sin() * pitch.cos(),
		);
		camera = Camera3D {
			position: grid_center + cam_offset,
			target: grid_center,
			up: vec3(0.0, 1.0, 0.0),
			..Default::default()
		};

		// ── Draw scene ─────────────────────────────────────────────────────────
		clear_background(Color::from_rgba(22, 22, 32, 255));
		set_camera(&camera);

		for x in 0..8usize {
			for y in 0..8usize {
				for z in 0..3usize {
					if let Some(variant) = cubes[x][y][z] {
						let pos = vec3(x as f32 * step, z as f32 * step, y as f32 * step);
						let sz = vec3(cube_size, cube_size, cube_size);
						draw_cube(pos, sz, None, if variant { color_b } else { color_a });
						draw_cube_wires_thick(pos, sz, Color::from_rgba(0, 0, 0, 255), 0.02);
					}
				}
			}
		}

		// ── 2D overlay ─────────────────────────────────────────────────────────
		set_default_camera();

		let floor = -step * 0.5;
		let lo = -margin;
		let hi = 7.0 * step + margin;

		for i in 0..8usize {
			let wx = i as f32 * step;
			draw_label(
				&i.to_string(),
				vec3(wx, floor, lo),
				&camera,
				lbl_color,
				lsize,
			);
			draw_label(
				&i.to_string(),
				vec3(wx, floor, hi),
				&camera,
				lbl_color,
				lsize,
			);
		}
		draw_label(
			"x",
			vec3(3.5 * step, floor, lo - step * 0.7),
			&camera,
			lbl_x,
			asize,
		);
		draw_label(
			"x",
			vec3(3.5 * step, floor, hi + step * 0.7),
			&camera,
			lbl_x,
			asize,
		);

		for i in 0..8usize {
			let wz = i as f32 * step;
			draw_label(
				&i.to_string(),
				vec3(lo, floor, wz),
				&camera,
				lbl_color,
				lsize,
			);
			draw_label(
				&i.to_string(),
				vec3(hi, floor, wz),
				&camera,
				lbl_color,
				lsize,
			);
		}
		draw_label(
			"y",
			vec3(lo - step * 0.7, floor, 3.5 * step),
			&camera,
			lbl_y,
			asize,
		);
		draw_label(
			"y",
			vec3(hi + step * 0.7, floor, 3.5 * step),
			&camera,
			lbl_y,
			asize,
		);

		let corners = [(lo, lo), (hi, lo), (lo, hi), (hi, hi)];
		for i in 0..3usize {
			let wy = i as f32 * step;
			for &(cx, cz) in &corners {
				draw_label(&i.to_string(), vec3(cx, wy, cz), &camera, lbl_color, lsize);
			}
		}
		for &(cx, cz) in &corners {
			draw_label("z", vec3(cx, 3.2 * step, cz), &camera, lbl_z, asize);
		}

		// ── Static UI ──────────────────────────────────────────────────────────
		draw_text("Hello, world!", 14.0, 36.0, 34.0, WHITE);

		if now.elapsed() >= fps_update_interval {
			fps = get_fps();
			now = std::time::Instant::now();
		}
		draw_fps(fps);

		let btn_y = screen_height() - 44.0;
		let mut x_cursor = screen_width() - 34.0f32;
		x_cursor -= measure_text("Delete random cube [X]", None, btn_size as u16, 1.0).width;

		if button("Delete random cube [X]", x_cursor, btn_y, btn_size) && !drag_moved {
			delete_one = true;
			dragging = false;
		}

		let (pause_clicked, quit_clicked) = draw_pause_quit_buttons(paused, btn_y, btn_size);
		let _ = x_cursor; // suppress warning
		if pause_clicked {
			paused = true;
		}
		if quit_clicked {
			break;
		}

		// ── Delete logic ───────────────────────────────────────────────────────
		if delete_one {
			let alive: Vec<(usize, usize, usize)> = (0..8)
				.flat_map(|x| (0..8).flat_map(move |y| (0..3).map(move |z| (x, y, z))))
				.filter(|&(x, y, z)| cubes[x][y][z].is_some())
				.collect();
			if !alive.is_empty() {
				let (x, y, z) = alive[rand::gen_range(0, alive.len())];
				cubes[x][y][z] = None;
			}
		}

		// ── Minimap ────────────────────────────────────────────────────────────
		let map_px = 168.0f32;
		let cell_px = map_px / 8.0;
		let map_x = screen_width() - map_px - 14.0;
		let map_y = 14.0f32;
		let border = 3.0f32;

		draw_rectangle(
			map_x - border,
			map_y - border,
			map_px + border * 2.0,
			map_px + border * 2.0 + 20.0,
			Color::from_rgba(0, 0, 0, 200),
		);
		for x in 0..8usize {
			for y in 0..8usize {
				let top_col = (0..3)
					.rev()
					.find_map(|z| cubes[x][y][z].map(|v| if v { color_b } else { color_a }))
					.unwrap_or(Color::from_rgba(45, 45, 55, 255));
				draw_rectangle(
					map_x + x as f32 * cell_px,
					map_y + y as f32 * cell_px,
					cell_px - 1.5,
					cell_px - 1.5,
					top_col,
				);
			}
		}
		draw_text(
			"Top View",
			map_x,
			map_y + map_px + 17.0,
			16.0,
			Color::from_rgba(180, 180, 180, 255),
		);

		next_frame().await;
	}
}

// #![allow(dead_code)] // TODO: remove, it's just nice to quiet down rust-analyzer

// pub mod caminos;
// pub mod mcts;
// pub mod util;

// use std::{
// 	f64::consts::SQRT_2,
// 	io::{self, Write},
// };

// use crate::{
// 	caminos::{
// 		file::{ReadFromPath, WriteToPath},
// 		placement::PlacementRefs,
// 		state::{GameResult, GameState, Player},
// 	},
// 	mcts::{
// 		agent::{MctsAgent, MctsAgentConfig},
// 		policy::{
// 			action::RobustChild,
// 			computation::IterativeComputationalLimit,
// 			expansion::{ExpandAlways, ExpandRandomly},
// 			reward::RewardPolicy,
// 			rollout::RolloutRandomly,
// 			selection::Ucb1,
// 		},
// 	},
// 	util::ansi,
// };

// fn main() {
// 	let mut a = MctsAgent::new(MctsAgentConfig {
// 		computational_limit: Box::new(IterativeComputationalLimit {
// 			iterations: 100_000,
// 		}),
// 		reward_policy: RewardPolicy {
// 			strong_win: 1.0,
// 			weak_win: 0.8,
// 			draw: 0.5,
// 			weak_loss: -1.0,
// 			strong_loss: -1.0,
// 		},
// 		selection_policy: Box::new(Ucb1 {
// 			exploration_constant: SQRT_2,
// 		}),
// 		expansion_predicate: Box::new(ExpandAlways),
// 		expansion_policy: Box::new(ExpandRandomly::unseeded()),
// 		rollout_policy: Box::new(RolloutRandomly::unseeded()),
// 		action_policy: Box::new(RobustChild),
// 	});

// 	let mut b = MctsAgent::new(MctsAgentConfig {
// 		computational_limit: Box::new(IterativeComputationalLimit {
// 			iterations: 100_000,
// 		}),
// 		reward_policy: RewardPolicy {
// 			strong_win: 1.0,
// 			weak_win: 0.8,
// 			draw: 0.5,
// 			weak_loss: -1.0,
// 			strong_loss: -1.0,
// 		},
// 		selection_policy: Box::new(Ucb1 {
// 			exploration_constant: SQRT_2,
// 		}),
// 		expansion_predicate: Box::new(ExpandAlways),
// 		expansion_policy: Box::new(ExpandRandomly::unseeded()),
// 		rollout_policy: Box::new(RolloutRandomly::unseeded()),
// 		action_policy: Box::new(RobustChild),
// 	});

// 	let mut state = GameState::EMPTY;
// 	let mut placements: PlacementRefs = Vec::new();

// 	loop {
// 		if let Some(result) = state.determine_winner() {
// 			match result {
// 				GameResult::StrongWin(Player::A) => {
// 					println!("{}Player A wins strongly!{}", ansi::GREEN, ansi::RESET)
// 				}
// 				GameResult::WeakWin(Player::A) => {
// 					println!("{}Player A wins weakly!{}", ansi::BLUE, ansi::RESET)
// 				}

// 				GameResult::StrongWin(Player::B) => {
// 					println!("{}Player B wins strongly!{}", ansi::RED, ansi::RESET)
// 				}
// 				GameResult::WeakWin(Player::B) => {
// 					println!("{}Player B wins weakly!{}", ansi::RED, ansi::RESET)
// 				}

// 				GameResult::Draw => println!("{}It's a draw!{}", ansi::YELLOW, ansi::RESET),
// 			}

// 			break;
// 		}

// 		let best_move = match state.next_player {
// 			Player::A => a.find_best_placement(&state),
// 			Player::B => b.find_best_placement(&state),
// 		};

// 		if let Some(placement) = best_move {
// 			println!("Player {} places {}", state.next_player, placement);
// 			state.apply_placement(placement);
// 			placements.push(placement);
// 		} else {
// 			println!("Game over! No valid move found");
// 			break;
// 		}

// 		println!("{state}");

// 		if std::env::args().any(|arg| arg == "--wait") {
// 			print!("Press Enter to continue...");
// 			io::stdout().flush().unwrap();
// 			let mut input = String::new();
// 			io::stdin().read_line(&mut input).unwrap();
// 			println!();
// 		}
// 	}

// 	let path = "result.caminos";
// 	placements.write_to_path(path, true).unwrap();
// 	let loaded_state = GameState::read_from_path(path).unwrap();
// 	println!("\nLoaded game state from {}:\n{}", path, loaded_state);
// }
