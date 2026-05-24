// use ansi_to_tui::IntoText;
// use ratatui::{
// 	Frame,
// 	buffer::Buffer,
// 	layout::{Alignment, Constraint, Direction, Layout, Rect},
// 	widgets::{Block, Padding, Paragraph, Widget},
// };

// use crate::{caminos::board::BitBoard, mcts::agent::MctsAgent, ui::tui::display::PlacementPreview};

pub mod ansi;
pub mod display;

// pub fn run(_agent: &mut MctsAgent) {
// 	let mut term = ratatui::init();

// 	loop {
// 		term.draw(render).ok();
// 	}
// }

// fn render(frame: &mut Frame) {
// 	let layout = Layout::default()
// 		.direction(Direction::Vertical)
// 		.constraints([Constraint::Percentage(100)].as_ref())
// 		.split(frame.area());

// 	BitBoard::EMPTY.render(
// 		centered(layout[0], BitBoard::WIDTH, BitBoard::HEIGHT),
// 		frame.buffer_mut(),
// 	);
// }

// fn centered(area: Rect, width: u16, height: u16) -> Rect {
// 	let [_, vertical, _] = Layout::vertical([
// 		Constraint::Fill(1),
// 		Constraint::Length(height),
// 		Constraint::Fill(1),
// 	])
// 	.areas(area);

// 	let [_, horizontal, _] = Layout::horizontal([
// 		Constraint::Fill(1),
// 		Constraint::Length(width),
// 		Constraint::Fill(1),
// 	])
// 	.areas(vertical);

// 	horizontal
// }

// impl BitBoard {
// 	const CONTENT_WIDTH: u16 = 49;
// 	const CONTENT_HEIGHT: u16 = 9;

// 	pub const WIDTH: u16 = Self::CONTENT_WIDTH
//         + 2  // border
//         + 6; // padding

// 	pub const HEIGHT: u16 = Self::CONTENT_HEIGHT
// 	    + 2  // border
//         + 2; // padding
// }

// impl Widget for BitBoard {
// 	fn render(self, area: Rect, buf: &mut Buffer) {
// 		// should be exactly 49 x 9

// 		Paragraph::new(format!("{}", self).into_text().unwrap())
// 			.block(
// 				Block::bordered()
// 					.title(" BOARD ")
// 					.title_alignment(Alignment::Center)
// 					.padding(Padding::symmetric(3, 1)),
// 			)
// 			.render(area, buf);
// 	}
// }

// impl Widget for PlacementPreview<'_> {
// 	fn render(self, area: Rect, buf: &mut Buffer) {
// 		// 49 x 9 or
// 		// Paragraph::new(format!("{}", self).into_text().unwrap()).render(area, buf);

// 		Paragraph::new(format!("{}", self).into_text().unwrap())
// 			.block(
// 				Block::bordered()
// 					.title("Preview")
// 					.title_alignment(Alignment::Center),
// 			)
// 			.render(area, buf);
// 	}
// }
