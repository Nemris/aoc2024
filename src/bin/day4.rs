#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Orientation of a matrix.
enum Orientation {
    Rows,
    Columns,
}

/// Direction of a diagonal.
enum Direction {
    LeftToRight,
    RightToLeft,
}

/// An n*n matrix containing the haystack to examine.
struct SquareMatrix {
    /// Raw data.
    blob: Vec<char>,
    /// Length of a side of the matrix.
    width: usize,
}

impl SquareMatrix {
    /// Creates a new `SquareMatrix` from the data in `blob`.
    ///
    /// The square root of `blob`'s `.len()` must be an integer.
    fn new(blob: &[char]) -> Result<Self, &'static str> {
        // Pretty hacky, but passing correct data is on the caller.
        #[allow(clippy::cast_precision_loss)]
        let width = (blob.len() as f64).sqrt();
        if width.fract() != 0.0 {
            return Err("invalid matrix shape");
        }
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let width = width as usize;

        let blob = blob.to_vec();
        Ok(Self { blob, width })
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

        let row_range = 0..=(self.width - needle.len());
        let col_range = match direction {
            Direction::LeftToRight => 0..self.width + 1 - needle.len(),
            Direction::RightToLeft => needle.len() - 1..self.width,
        };

        let mut matches = 0;
        for y in row_range {
            for x in col_range.clone() {
                let w = get_diagonal(&rows[y..y + needle.len()], x, direction);
                if slices_match(&w, needle) {
                    matches += 1;
                }
            }
        }

        matches
    }

    /// Counts the occurrences of two diagonal `needle`s that intersect at their midpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if `needle`'s length is less than 3 or an even number.
    fn count_intersections(&self, needle: &[char]) -> Result<usize, &'static str> {
        if needle.len() < 3 || needle.len() % 2 == 0 {
            return Err("invalid needle length");
        }

        let midpoint = needle.len() / 2;

        let rows = self.rows();
        let row_range = needle[..midpoint].len()..self.width - needle[midpoint + 1..].len();
        let col_range = midpoint..self.width - midpoint;

        let mut matches = 0;
        for y in row_range {
            for x in col_range.clone() {
                if rows[y][x] != needle[midpoint] {
                    continue;
                }

                let rows = &rows[(y - midpoint)..=(y + midpoint)];
                let ltr_diag = get_diagonal(rows, x - midpoint, &Direction::LeftToRight);
                if !slices_match(&ltr_diag, needle) {
                    continue;
                }

                let rtl_diag = get_diagonal(rows, x + midpoint, &Direction::RightToLeft);
                if slices_match(&rtl_diag, needle) {
                    matches += 1;
                }
            }
        }

        Ok(matches)
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
    let matrix = SquareMatrix::new(&data)?;

    let needle = "XMAS".chars().collect::<Vec<_>>();
    println!("Occurrences in matrix: {}", matrix.count_in_matrix(&needle));

    let needle = "MAS".chars().collect::<Vec<_>>();
    println!("Intersections: {}", matrix.count_intersections(&needle)?);

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
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count(&Orientation::Rows, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_cols() {
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count(&Orientation::Columns, &needle), 3);
    }

    #[test]
    fn square_matrix_finds_needle_in_ltr_diagonals() {
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_diagonals(&Direction::LeftToRight, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_rtl_diagonals() {
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_diagonals(&Direction::RightToLeft, &needle), 5);
    }

    #[test]
    fn square_matrix_finds_needle_in_self() {
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "XMAS".chars().collect();

        assert_eq!(sm.count_in_matrix(&needle), 18);
    }

    #[test]
    fn square_matrix_finds_intersected_needle_in_self() {
        let sm = SquareMatrix::new(&get_test_data()).unwrap();
        let needle: Vec<char> = "MAS".chars().collect();

        assert_eq!(sm.count_intersections(&needle).unwrap(), 9);
    }
}
