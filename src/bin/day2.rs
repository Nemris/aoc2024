#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;

type Level = u32;

/// A report containing measurement levels.
#[derive(Clone, Debug, PartialEq)]
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
    /// Creates a new Report from the levels in `self`, excluding the one at `pos`.
    fn new_excluding_level(&self, pos: usize) -> Self {
        let v = self
            .0
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != pos)
            .map(|(_, v)| *v)
            .collect();
        Self(v)
    }

    /// Tries to build a safe Report if `self` is unsafe, by removing up to one level.
    fn try_dampen(&self) -> Result<Self, &'static str> {
        if self.is_safe() {
            return Ok(self.clone());
        }

        for (i, _) in self.0.iter().enumerate() {
            let r = self.new_excluding_level(i);
            if r.is_safe() {
                return Ok(r);
            }
        }

        // The Problem Dampener failed, nothing can be done.
        Err("cannot correct report error")
    }

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

    let safe = reports.iter().filter(|r| r.is_safe()).count();
    let dampened = reports
        .iter()
        .filter(|r| !r.is_safe())
        .flat_map(Report::try_dampen)
        .count();
    println!("Safe reports: {safe}",);
    println!("Dampened reports: {dampened}");
    println!("Total safe reports: {}", safe + dampened);

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

    #[test]
    fn report_dampener_changes_nothing_if_report_is_safe() {
        let r = Report(vec![7, 6, 4, 2, 1]);
        assert!(r.try_dampen().is_ok_and(|r2| r2 == r));

        let r = Report(vec![1, 3, 6, 7, 9]);
        assert!(r.try_dampen().is_ok_and(|r2| r2 == r));
    }

    #[test]
    fn report_dampener_removes_one_bad_level() {
        let r = Report(vec![1, 3, 2, 4, 5]);
        assert!(r.try_dampen().is_ok_and(|r| r.is_safe()));

        let r = Report(vec![8, 6, 4, 4, 1]);
        assert!(r.try_dampen().is_ok_and(|r| r.is_safe()));
    }

    #[test]
    fn report_dampener_bails_if_more_bad_levels() {
        let r = Report(vec![1, 2, 7, 8, 9]);
        assert!(r.try_dampen().is_err());

        let r = Report(vec![9, 7, 6, 2, 1]);
        assert!(r.try_dampen().is_err());
    }
}
