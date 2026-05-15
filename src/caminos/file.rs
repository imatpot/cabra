use std::{
	fs,
	io::{self, Write},
};

use crate::caminos::{
	placement::{LEGAL_PLACEMENTS, Placement, Position},
	state::GameState,
};

/// Reading from a file at a given path.
pub trait ReadFromPath: Sized {
	/// Reads an instance of the type from a text file at the given path.
	/// Supports trailing comments using `#` and ignores non-digit characters.
	///
	/// Returns an error if
	/// any line does not contain exactly 12 digits,
	/// any digit is out of the valid range (0-7 for x and y, 0-2 for z),
	/// or if any resulting placement is invalid (e.g. cells not connected).
	fn read_from_path(path: &str) -> io::Result<Self>;
}

/// Writing to a file at a given path.
pub trait WriteToPath: Sized {
	/// Writes the instance to a text file at the given path,
	/// overwriting any existing file.
	///
	/// If `notation_comment` is true, appends a comment with the human-readable
	/// notation of the placement after the 12-digit string, separated by `#`.
	fn write_to_path(&self, path: &str, notation_comment: bool) -> io::Result<()>;

	/// Appends the instance to a text file at the given path,
	/// creating the file if it does not exist.
	///
	/// If `notation_comment` is true, appends a comment with the human-readable
	/// notation of the placement after the 12-digit string, separated by `#`.
	fn append_to_path(&self, path: &str, notation_comment: bool) -> io::Result<()>;
}

impl ReadFromPath for GameState {
	fn read_from_path(path: &str) -> io::Result<Self> {
		Ok(GameState::from(
			Vec::<&'static Placement>::read_from_path(path)?.as_slice(),
		))
	}
}

impl ReadFromPath for Vec<&'static Placement> {
	fn read_from_path(path: &str) -> io::Result<Self> {
		fs::read_to_string(path)?
			.lines()
			.filter(|line| !line.trim().is_empty())
			.map(|line| Placement::try_ref_from_twelve_digits_string(line))
			.collect()
	}
}

impl WriteToPath for Vec<&'static Placement> {
	fn write_to_path(&self, path: &str, notation_comment: bool) -> io::Result<()> {
		let mut file = fs::File::create(path)?;

		for placement in self {
			let line = if notation_comment {
				format!(
					"{}   # {}\n",
					to_twelve_digits_string(placement.occupied_positions),
					placement
				)
			} else {
				format!(
					"{}\n",
					to_twelve_digits_string(placement.occupied_positions)
				)
			};

			file.write_all(line.as_bytes())?;
		}

		Ok(())
	}

	fn append_to_path(&self, path: &str, notation_comment: bool) -> io::Result<()> {
		let mut file = fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)?;

		for placement in self {
			let line = if notation_comment {
				format!(
					"{}   # {}\n",
					to_twelve_digits_string(placement.occupied_positions),
					placement
				)
			} else {
				format!(
					"{}\n",
					to_twelve_digits_string(placement.occupied_positions)
				)
			};

			file.write_all(line.as_bytes())?;
		}

		Ok(())
	}
}

impl Placement {
	/// Parses a string containing 12 digits representing four occupied cells,
	/// each cell is encoded with its x, y, and z coordinates.
	///
	/// Supports trailing comments using `#` and ignores non-digit characters.
	///
	/// Returns an error if
	/// the string does not contain exactly 12 digits,
	/// any digit is out of the valid range (0-7 for x and y, 0-2 for z),
	/// or the resulting placement is invalid (e.g. cells not connected).
	fn try_ref_from_twelve_digits_string(string: &str) -> io::Result<&'static Placement> {
		let stripped_comment = string.split('#').next().unwrap_or("");
		let digit_values: Vec<u8> = stripped_comment
			.as_bytes()
			.iter()
			.filter(|b| b.is_ascii_digit())
			.map(|b| b - b'0')
			.collect();

		if digit_values.len() != 12 {
			return io_err(
				io::ErrorKind::InvalidData,
				format!(
					"Expected string containign exactly 12 digits, found \"{}\"",
					digit_values.len()
				),
			);
		}

		let mut coordinates = [(0, 0, 0); 4];

		for (i, point) in digit_values.chunks(3).enumerate() {
			let x = point[0];
			let y = point[1];
			let z = point[2];

			if x > 7 || y > 7 || z > 2 {
				return io_err(
					io::ErrorKind::InvalidData,
					format!(
						"Digit values must be in the range 0-7 for x and y, and 0-2 for z. Found \"{}{}{}\"",
						x, y, z
					),
				);
			}

			coordinates[i] = (x, y, z);
		}

		LEGAL_PLACEMENTS
			.all()
			.find(|placement| placement.occupied_positions == coordinates)
			.ok_or_else(|| {
				io_err::<Placement>(
					io::ErrorKind::InvalidData,
					format!(
						"No valid placement found for coordinates \"{}\"",
						to_twelve_digits_string(coordinates)
					),
				)
				.err()
				.unwrap()
			})
	}
}

/// Thin wrapper around [`std::io::Error::new`].
fn io_err<T>(kind: io::ErrorKind, message: String) -> io::Result<T> {
	Err(io::Error::new(kind, message))
}

/// Prints a set of 4 [`Position`]s into the format `XYZ XYZ XYZ XYZ`.
fn to_twelve_digits_string(coordinates: [Position; 4]) -> String {
	format!(
		"{}{}{} {}{}{} {}{}{} {}{}{}",
		coordinates[0].0,
		coordinates[0].1,
		coordinates[0].2,
		coordinates[1].0,
		coordinates[1].1,
		coordinates[1].2,
		coordinates[2].0,
		coordinates[2].1,
		coordinates[2].2,
		coordinates[3].0,
		coordinates[3].1,
		coordinates[3].2,
	)
}
