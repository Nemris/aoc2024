#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::slice::Iter;
use std::str::FromStr;

/// A printing rule.
#[derive(Debug)]
struct Rule {
    /// This number must come before `y` in a valid `Update`.
    x: u32,
    /// This number must come after `x` in a valid `Update`.
    y: u32,
}

impl Rule {
    /// Determines if `update` is valid based on `self`.
    fn is_valid(&self, update: &Update) -> bool {
        let x_pos = update.iter().position(|&n| n == self.x);
        let y_pos = update.iter().position(|&n| n == self.y);
        match (x_pos, y_pos) {
            (Some(x), Some(y)) => x < y,
            _ => true,
        }
    }
}

impl FromStr for Rule {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .splitn(2, '|')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            x: parts[0],
            y: parts[1],
        })
    }
}

/// A manual page update.
#[derive(Debug)]
struct Update(Vec<u32>);

impl FromStr for Update {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(parts))
    }
}

impl Update {
    /// Determines if `self` is valid based on a series of `rules`.
    fn is_valid(&self, rules: &[Rule]) -> bool {
        rules.iter().all(|r| r.is_valid(self))
    }

    /// Applies all the `rules` until `self` is corrected.
    fn apply_all(&mut self, rules: &[Rule]) {
        // NOTE: very slow.
        while let Some(r) = self.next_failed_rule(rules) {
            self.apply(r);
        }
    }

    /// Applies a single `rule` to correct `self`.
    fn apply(&mut self, rule: &Rule) {
        let x_pos = self
            .0
            .iter()
            .position(|&n| n == rule.x)
            .expect("the position should exist");
        let y_pos = self
            .0
            .iter()
            .position(|&n| n == rule.y)
            .expect("the position should exist");
        self.0.swap(x_pos, y_pos);
    }

    /// Returns the page number at `self`'s middle.
    fn middle_page(&self) -> u32 {
        self.0[self.0.len() / 2]
    }

    /// Returns the next rule that `self` fails to respect.
    fn next_failed_rule<'r>(&self, rules: &'r [Rule]) -> Option<&'r Rule> {
        rules.iter().find(|r| !r.is_valid(self))
    }

    /// Returns an iterator over `self`'s pages.
    fn iter(&self) -> Iter<'_, u32> {
        self.0.iter()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut rules = vec![];
    let mut updates = vec![];
    for line in reader.lines() {
        let line = line?;
        if line.find('|').is_some() {
            rules.push(Rule::from_str(&line)?);
        } else if line.find(',').is_some() {
            updates.push(Update::from_str(&line)?);
        }
    }

    let total: u32 = updates
        .iter()
        .filter(|u| u.is_valid(&rules))
        .map(Update::middle_page)
        .sum();
    println!("Sum of middle pages: {total}");

    let mut bad_updates: Vec<Update> = updates
        .into_iter()
        .filter(|u| !u.is_valid(&rules))
        .collect();
    for u in &mut bad_updates {
        u.apply_all(&rules);
    }
    let total: u32 = bad_updates.iter().map(Update::middle_page).sum();
    println!("Sum of middle pages (fixed updates): {total}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_rules() -> Vec<Rule> {
        let data = vec![
            "47|53", "97|13", "97|61", "97|47", "75|29", "61|13", "75|53", "29|13", "97|29",
            "53|29", "61|53", "97|53", "61|29", "47|13", "75|47", "97|75", "47|61", "75|61",
            "47|29", "75|13", "53|13",
        ];
        data.into_iter()
            .map(Rule::from_str)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn get_test_updates() -> Vec<Update> {
        let data = vec![
            "75,47,61,53,29",
            "97,61,53,29,13",
            "75,29,13",
            "75,97,47,61,53",
            "61,13,29",
            "97,13,75,29,47",
        ];
        data.into_iter()
            .map(Update::from_str)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn valid_updates_pass_verification() {
        let rules = get_test_rules();
        let updates = get_test_updates();

        assert!(updates[0].is_valid(&rules));
        assert!(updates[1].is_valid(&rules));
        assert!(updates[2].is_valid(&rules));
        assert!(!updates[3].is_valid(&rules));
        assert!(!updates[4].is_valid(&rules));
        assert!(!updates[5].is_valid(&rules));
    }

    #[test]
    fn valid_updates_evaluate_to_correct_value() {
        let rules = get_test_rules();
        let updates: Vec<Update> = get_test_updates()
            .into_iter()
            .filter(|u| u.is_valid(&rules))
            .collect();
        let total: u32 = updates.iter().map(Update::middle_page).sum();

        assert_eq!(total, 143);
    }

    #[test]
    fn fixed_updates_evaluate_to_correct_value() {
        let rules = get_test_rules();
        let mut updates: Vec<Update> = get_test_updates()
            .into_iter()
            .filter(|u| !u.is_valid(&rules))
            .collect();

        for u in updates.iter_mut() {
            u.apply_all(&rules);
        }
        let total: u32 = updates.iter().map(Update::middle_page).sum();
        assert_eq!(total, 123);
    }
}
