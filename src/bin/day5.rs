#![warn(clippy::pedantic)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

/// Rules to sort page updates with.
///
/// Each page X is mapped to all the pages Y that must come after it.
#[derive(Debug)]
struct PageRules(HashMap<u32, Vec<u32>>);

impl PageRules {
    /// Returns a new `PageRules`.
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Parses a new `rule` and inserts it in `self`.
    fn insert(&mut self, rule: &str) -> Result<(), ParseIntError> {
        let parts = rule
            .splitn(2, '|')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        self.0
            .entry(parts[0])
            .and_modify(|v| v.push(parts[1]))
            .or_insert(vec![parts[1]]);
        Ok(())
    }

    /// Returns the pages that must come after a page `x`.
    fn get(&self, x: u32) -> Option<&Vec<u32>> {
        self.0.get(&x)
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
    /// Sorts this update according to `rules`.
    fn sort(&mut self, rules: &PageRules) {
        self.0.sort_unstable_by(|x, y| match rules.get(*x) {
            Some(ys) => {
                if ys.contains(y) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            None => Ordering::Equal,
        });
    }

    /// Checks if the pages in this update are sorted according to `rules`.
    fn is_sorted(&self, rules: &PageRules) -> bool {
        self.0.is_sorted_by(|x, y| match rules.get(*y) {
            Some(xs) => !xs.contains(x),
            None => true,
        })
    }

    /// Returns the page number at `self`'s middle.
    fn middle_page(&self) -> u32 {
        self.0[self.0.len() / 2]
    }
}

/// Sums the middle pages of `updates`.
fn sum_middle_pages<'a, I>(updates: I) -> u32
where
    I: IntoIterator<Item = &'a Update>,
{
    updates.into_iter().map(Update::middle_page).sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut rules = PageRules::new();
    let mut updates = vec![];
    for line in reader.lines() {
        let line = line?;
        if line.find('|').is_some() {
            rules.insert(&line)?;
        } else if line.find(',').is_some() {
            updates.push(Update::from_str(&line)?);
        }
    }

    let sorted = updates.iter().filter(|u| u.is_sorted(&rules));
    println!("Sum of middle pages: {}", sum_middle_pages(sorted));

    let sorted = {
        let mut unsorted: Vec<_> = updates
            .into_iter()
            .filter(|u| !u.is_sorted(&rules))
            .collect();
        for u in &mut unsorted {
            u.sort(&rules);
        }
        unsorted
    };
    println!(
        "Sum of middle pages (after sorting): {}",
        sum_middle_pages(&sorted)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_rules() -> PageRules {
        let data = vec![
            "47|53", "97|13", "97|61", "97|47", "75|29", "61|13", "75|53", "29|13", "97|29",
            "53|29", "61|53", "97|53", "61|29", "47|13", "75|47", "97|75", "47|61", "75|61",
            "47|29", "75|13", "53|13",
        ];
        let mut pr = PageRules::new();
        for r in data {
            pr.insert(r).unwrap();
        }
        pr
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

        assert!(updates[0].is_sorted(&rules));
        assert!(updates[1].is_sorted(&rules));
        assert!(updates[2].is_sorted(&rules));
        assert!(!updates[3].is_sorted(&rules));
        assert!(!updates[4].is_sorted(&rules));
        assert!(!updates[5].is_sorted(&rules));
    }

    #[test]
    fn valid_updates_evaluate_to_correct_value() {
        let rules = get_test_rules();
        let updates: Vec<Update> = get_test_updates()
            .into_iter()
            .filter(|u| u.is_sorted(&rules))
            .collect();

        assert_eq!(sum_middle_pages(&updates), 143);
    }

    #[test]
    fn sorted_updates_evaluate_to_correct_value() {
        let rules = get_test_rules();
        let mut updates: Vec<Update> = get_test_updates()
            .into_iter()
            .filter(|u| !u.is_sorted(&rules))
            .collect();

        for u in &mut updates {
            u.sort(&rules);
        }

        assert_eq!(sum_middle_pages(&updates), 123);
    }
}
