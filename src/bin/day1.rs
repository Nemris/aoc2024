#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;

/// Tries to convert a space-separated &str representing columns of integers to a Vec<u32>.
fn to_vec_int(s: &str) -> Result<Vec<u32>, ParseIntError> {
    s.split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    // Read lines and interpret as columns, then sort.
    let mut first_col: Vec<u32> = vec![];
    let mut second_col: Vec<u32> = vec![];
    for line in reader.lines() {
        let pair = to_vec_int(&line?)?;
        first_col.push(pair[0]);
        second_col.push(pair[1]);
    }
    first_col.sort_unstable();
    second_col.sort_unstable();

    // Compute distance.
    let distance: u32 = first_col
        .into_iter()
        .zip(second_col)
        .map(|(d1, d2)| d1.abs_diff(d2))
        .sum();
    println!("Distance: {distance}");

    Ok(())
}
