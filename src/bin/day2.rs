#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;

type Level = u32;

/// A report containing measurement levels.
#[derive(Debug)]
struct Report(Vec<Level>);

impl TryFrom<&str> for Report {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v = value
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(v))
    }
}

impl Report {
    /// Checks if this report's levels match safety rules.
    fn is_safe(&self) -> bool {
        if !self.is_sorted() {
            return false;
        }

        self.0.windows(2).all(|w| {
            let d = w[0].abs_diff(w[1]);
            d > 0 && d < 4
        })
    }

    /// Checks if this report is sorted in either ascending or descending order.
    fn is_sorted(&self) -> bool {
        self.0.is_sorted() || self.0.is_sorted_by(|a, b| a >= b)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut reports = vec![];
    for line in reader.lines() {
        let rep = Report::try_from(&*line?)?;
        reports.push(rep);
    }

    println!(
        "Safe reports: {}",
        reports.iter().filter(|r| r.is_safe()).count()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_passes_sample_tests() {
        let r = Report::try_from("7 6 4 2 1").unwrap();
        assert!(r.is_safe());

        let r = Report::try_from("1 2 7 8 9").unwrap();
        assert!(!r.is_safe());

        let r = Report::try_from("9 7 6 2 1").unwrap();
        assert!(!r.is_safe());

        let r = Report::try_from("1 3 2 4 5").unwrap();
        assert!(!r.is_safe());

        let r = Report::try_from("8 6 4 4 1").unwrap();
        assert!(!r.is_safe());

        let r = Report::try_from("1 3 6 7 9").unwrap();
        assert!(r.is_safe());
    }
}
