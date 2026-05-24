use crate::{
	caminos::{board::BitBoard, placement::Placement, state::GameState},
	ui::tui::ansi,
};

pub const PLAYER_A_COLOR: &'static str = ansi::MAGENTA;
pub const PLAYER_B_COLOR: &'static str = ansi::RESET;
pub const PREVIEW_COLOR: &'static str = ansi::GREEN;

impl std::fmt::Display for BitBoard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// Unoccupied: Alternate between dim ░▒ with flipped order on each row
		// Occupied: Regular █

		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let mut color = ansi::DIM;

					let char = if self.is_xyz_occupied(x, y, z) {
						color = ansi::RESET;
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{}{}{} ", color, char, ansi::RESET)?;
				}

				if z < 2 {
					write!(f, " ")?;
				}
			}

			writeln!(f)?;
		}

		Ok(())
	}
}

impl std::fmt::Display for GameState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let mut color = ansi::DIM;

					let char = if self.players[0].occupancy.is_xyz_occupied(x, y, z) {
						color = PLAYER_A_COLOR;
						'█'
					} else if self.players[1].occupancy.is_xyz_occupied(x, y, z) {
						color = PLAYER_B_COLOR;
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{}{}{} ", color, char, ansi::RESET)?;
				}

				if z < 2 {
					write!(f, " ")?;
				}
			}

			writeln!(f)?;
		}

		Ok(())
	}
}

pub struct PlacementPreview<'a>(pub &'a GameState, pub Option<&'static Placement>);

impl std::fmt::Display for PlacementPreview<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (state, placement) = (self.0, self.1);
		writeln!(f, "Layer 0          Layer 1          Layer 2")?;

		for y in 0..8 {
			for z in 0..3 {
				for x in 0..8 {
					let mut color = ansi::DIM;

					let char = if placement.is_some_and(|p| p.board_mask.is_xyz_occupied(x, y, z)) {
						color = PREVIEW_COLOR;
						'█'
					} else if state.players[0].occupancy.is_xyz_occupied(x, y, z) {
						color = PLAYER_A_COLOR;
						'█'
					} else if state.players[1].occupancy.is_xyz_occupied(x, y, z) {
						color = PLAYER_B_COLOR;
						'█'
					} else if y % 2 == 0 {
						if x % 2 == 0 { '░' } else { '▒' }
					} else {
						if x % 2 == 0 { '▒' } else { '░' }
					};

					write!(f, "{}{}{} ", color, char, ansi::RESET)?;
				}

				if z < 2 {
					write!(f, " ")?;
				}
			}

			writeln!(f)?;
		}

		Ok(())
	}
}
