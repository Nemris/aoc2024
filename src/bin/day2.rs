#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;

type Level = u32;

#[allow(dead_code)]
/// A report containing either safe or unsafe measurement levels.
#[derive(Debug)]
enum Report {
    Safe(SafeReport),
    Unsafe(UnsafeReport),
}

impl TryFrom<&str> for Report {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v = value
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?;

        if are_levels_safe(&v) {
            return Ok(Self::Safe(SafeReport(v)));
        }
        Ok(Self::Unsafe(UnsafeReport(v)))
    }
}

#[allow(dead_code)]
/// A report containing safe levels.
#[derive(Debug)]
struct SafeReport(Vec<Level>);

/// A report containing unsafe levels.
#[derive(Debug)]
struct UnsafeReport(Vec<Level>);

impl UnsafeReport {
    /// Tries to build a `SafeReport` by removing up to one level.
    fn try_dampen(&self) -> Result<SafeReport, &'static str> {
        for (i, _) in self.0.iter().enumerate() {
            let v = self.exclude_level(i);
            if are_levels_safe(&v) {
                return Ok(SafeReport(v));
            }
        }

        // The Problem Dampener failed, nothing can be done.
        Err("cannot correct report error")
    }

    /// Returns the levels in `self`, excluding the one at `pos`.
    fn exclude_level(&self, pos: usize) -> Vec<Level> {
        self.0
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != pos)
            .map(|(_, v)| *v)
            .collect()
    }
}

/// Checks if a slice of levels matches safety rules.
fn are_levels_safe(v: &[Level]) -> bool {
    if !v.is_sorted() && !v.is_sorted_by(|a, b| a >= b) {
        return false;
    }

    if v.windows(2)
        .any(|w| !(1..=3).contains(&w[0].abs_diff(w[1])))
    {
        return false;
    }

    true
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut reports = vec![];
    for line in reader.lines() {
        let rep = Report::try_from(&*line?)?;
        reports.push(rep);
    }

    let safe = reports
        .iter()
        .filter(|&r| match r {
            Report::Safe(_) => true,
            Report::Unsafe(_) => false,
        })
        .count();
    let dampened = reports
        .iter()
        .filter_map(|r| match r {
            Report::Unsafe(r) => Some(r.try_dampen()),
            Report::Safe(_) => None,
        })
        .flatten()
        .count();

    println!("Safe reports: {safe}",);
    println!("Dampened reports: {dampened}");
    println!("Total safe reports: {}", safe + dampened);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_LEVELS: &[&[Level]] = &[&[7, 6, 4, 2, 1], &[1, 3, 6, 7, 9]];
    const BAD_LEVELS: &[&[Level]] = &[&[1, 2, 7, 8, 9], &[9, 7, 6, 2, 1]];
    const CORRECTIBLE_LEVELS: &[&[Level]] = &[&[1, 3, 2, 4, 5], &[8, 6, 4, 4, 1]];

    fn stringify_levels(ls: &[Level]) -> String {
        ls.iter()
            .map(u32::to_string)
            .collect::<Vec<String>>()
            .join(" ")
    }

    #[test]
    fn report_from_str_succeeds() {
        let strings = GOOD_LEVELS
            .iter()
            .chain(BAD_LEVELS)
            .chain(CORRECTIBLE_LEVELS)
            .map(|ls| stringify_levels(ls));

        for s in strings {
            assert!(Report::try_from(&*s).is_ok());
        }
    }

    #[test]
    fn unsafereport_dampener_succeeds_for_correctible_levels() {
        for l in CORRECTIBLE_LEVELS {
            assert!(UnsafeReport(l.to_vec()).try_dampen().is_ok());
        }
    }

    #[test]
    fn unsafereport_dampener_fails_for_bad_levels() {
        for l in BAD_LEVELS {
            assert!(UnsafeReport(l.to_vec()).try_dampen().is_err());
        }
    }

    #[test]
    fn level_safety_check_succeeds_for_good_levels() {
        for l in GOOD_LEVELS {
            assert!(are_levels_safe(l));
        }
    }

    #[test]
    fn level_safety_check_fails_for_bad_levels() {
        for l in BAD_LEVELS.iter().chain(CORRECTIBLE_LEVELS) {
            assert!(!are_levels_safe(l));
        }
    }
}
