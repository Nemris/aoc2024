#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

enum Orientation {
    Rows,
    Columns,
}

enum Direction {
    LeftToRight,
    RightToLeft,
}

struct SquareMatrix {
    blob: Vec<char>,
    width: usize,
}

impl SquareMatrix {
    fn new(blob: &[char]) -> Self {
        // Pretty hacky, but we expect the caller to not mess with us.
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let width = (blob.len() as f64).sqrt() as usize;
        let blob = blob.to_vec();
        Self { blob, width }
    }

    fn count_in_matrix(&self, needle: &[char]) -> usize {
        self.count(&Orientation::Rows, needle)
            + self.count(&Orientation::Columns, needle)
            + self.count_in_diagonals(&Direction::LeftToRight, needle)
            + self.count_in_diagonals(&Direction::RightToLeft, needle)
    }

    /// Counts the occurrences of `needle` in self's rows or columns.
    ///
    /// Matches will also be counted if `needle` matches backwards.
    fn count(&self, orientation: &Orientation, needle: &[char]) -> usize {
        let haystack = match orientation {
            Orientation::Rows => self.rows(),
            Orientation::Columns => self.cols(),
        };

        let mut matches = 0;
        for h in haystack {
            for w in h.windows(needle.len()) {
                if slices_match(w, needle) {
                    matches += 1;
                }
            }
        }

        matches
    }

    /// Counts the occurrences of `needle` in self's diagonals following `direction`.
    ///
    /// Matches will also be counted if `needle` matches backwards.
    fn count_in_diagonals(&self, direction: &Direction, needle: &[char]) -> usize {
        let rows = self.rows();

        let first_row = 0;
        let last_row = self.width - needle.len() + 1;
        let first_char = match direction {
            Direction::LeftToRight => 0,
            Direction::RightToLeft => needle.len() - 1,
        };
        let last_char = match direction {
            Direction::LeftToRight => self.width - needle.len() + 1,
            Direction::RightToLeft => self.width,
        };

        let mut matches = 0;
        for i in first_row..last_row {
            for j in first_char..last_char {
                let w = get_diagonal(&rows[i..i + needle.len()], j, direction);
                if slices_match(&w, needle) {
                    matches += 1;
                }
            }
        }

        matches
    }

    /// Returns the rows in `self`.
    fn rows(&self) -> Vec<Vec<char>> {
        self.blob
            .chunks_exact(self.width)
            .map(<[char]>::to_vec)
            .collect()
    }

    /// Returns the columns in `self`.
    fn cols(&self) -> Vec<Vec<char>> {
        let mut cols = Vec::with_capacity(self.width);

        for row_idx in 0..self.width {
            // Skip to the first entry in a column, then collect it.
            let col: Vec<char> = self
                .blob
                .iter()
                .skip(row_idx)
                .step_by(self.width)
                .copied()
                .collect();
            cols.push(col);
        }
        cols
    }
}

/// Gets the diagonal starting from `start` and following `direction`.
fn get_diagonal(rows: &[Vec<char>], start: usize, direction: &Direction) -> Vec<char> {
    let mut diag = Vec::with_capacity(rows.len());

    for (i, row) in rows.iter().enumerate() {
        match direction {
            Direction::LeftToRight => diag.push(row[start + i]),
            Direction::RightToLeft => diag.push(row[start - i]),
        }
    }

    diag
}

/// Determines if `first` matches `second`, either normally or backwards.
fn slices_match(first: &[char], second: &[char]) -> bool {
    if first == second {
        return true;
    }
    first.iter().zip(second.iter().rev()).all(|(a, b)| a == b)
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let data = fs::read_to_string(dataset)?
        .chars()
        .filter(|&c| c != '\n')
        .collect::<Vec<_>>();

    let needle = "XMAS".chars().collect::<Vec<_>>();
    let matrix = SquareMatrix::new(&data);

    println!("Occurrences in matrix: {}", matrix.count_in_matrix(&needle));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<char> {
        let test_data = vec![
            "MMMSXXMASM",
            "MSAMXMSMSA",
            "AMXSXMAAMM",
            "MSAMASMSMX",
            "XMASAMXAMM",
            "XXAMMXXAMA",
            "SMSMSASXSS",
            "SAXAMASAAA",
            "MAMMMXMMMM",
            "MXMXAXMASX",
        ];
        test_data.into_iter().flat_map(|s| s.chars()).collect()
    }

    #[test]
    fn square_matrix_finds_needle_in_rows() {
        let sm = SquareMatrix::new(&get_test_data());
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count(&Orientation::Rows, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_cols() {
        let sm = SquareMatrix::new(&get_test_data());
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count(&Orientation::Columns, &needle), 3);
    }

    #[test]
    fn square_matrix_finds_needle_in_ltr_diagonals() {
        let sm = SquareMatrix::new(&get_test_data());
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_diagonals(&Direction::LeftToRight, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_rtl_diagonals() {
        let sm = SquareMatrix::new(&get_test_data());
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_diagonals(&Direction::RightToLeft, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_self() {
        let sm = SquareMatrix::new(&get_test_data());
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_matrix(&needle), 18);
    }
}