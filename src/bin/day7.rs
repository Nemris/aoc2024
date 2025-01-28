#![warn(clippy::pedantic)]
#![allow(dead_code)]

use std::error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

/// Possible errors for this program.
#[derive(Debug)]
enum Error {
    /// The equation is too short.
    EquationTooShort,
    /// An equation operand is malformed.
    MalformedOperand(ParseIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EquationTooShort => write!(f, "equation too short"),
            Error::MalformedOperand(e) => write!(f, "malformed operand: {e}"),
        }
    }
}

impl error::Error for Error {}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::MalformedOperand(e)
    }
}

/// An equation with a result and some values.
#[derive(Debug)]
struct Equation {
    /// The expected result of this equation.
    result: u64,
    /// Values that should evaluate to `result`.
    values: Vec<u64>,
}

impl FromStr for Equation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace(':', "");
        let mut parts = s.split(' ').map(str::parse);
        let result = parts.next().ok_or(Error::EquationTooShort)??;
        let values = parts.collect::<Result<_, _>>()?;

        Ok(Self { result, values })
    }
}

impl Equation {
    /// Determines if the values in `self` can produce its result.
    fn is_valid(&self) -> bool {
        if (self.values.is_empty() && self.result == 1)
            || (self.values.len() == 1 && self.result == self.values[0])
        {
            return true;
        }

        let mut total = self.result;
        for (i, v) in self.values.iter().rev().enumerate() {
            if *v > total {
                return false;
            }

            if total % v == 0 {
                // Since `v` is a divisor, let's try that possible path first.
                let sub_eq = Equation {
                    result: total / v,
                    values: self.values[..self.values.len() - (i + 1)].to_vec(),
                };
                if sub_eq.is_valid() {
                    return true;
                }
            }

            if let Some(n) = disjoin(total, *v) {
                // Since `total` could be disjoined, let's try this path too.
                let sub_eq = Equation {
                    result: n,
                    values: self.values[..self.values.len() - (i + 1)].to_vec(),
                };
                if sub_eq.is_valid() {
                    return true;
                }
            }

            // Last ditch attempt to validate by using `v` as subtrahend.
            total -= *v;
        }

        false
    }
}

/// Disjoins `y` from `x` and returns the result.
///
/// If `y` is zero, less than `x` or not concatenated to `x`, `None` is returned.
///
/// # Examples
///
/// ```
/// assert_eq!(disjoin(1234, 34), Some(12));
/// ```
fn disjoin(x: u64, y: u64) -> Option<u64> {
    let digits = y.checked_ilog10()? + 1;
    let divisor = 10u64.pow(digits);
    if y > x || x % divisor != y {
        return None;
    }

    Some(x / divisor)
}

/// Sums the results of `equations`.
fn sum_results<I>(equations: I) -> u64
where
    I: IntoIterator<Item = Equation>,
{
    equations.into_iter().map(|e| e.result).sum()
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut eqs = vec![];
    for line in reader.lines() {
        eqs.push(Equation::from_str(&line?)?);
    }

    let valid_eqs: Vec<_> = eqs.into_iter().filter(Equation::is_valid).collect();
    let total = sum_results(valid_eqs);
    println!("Total calibration result: {total}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_equations() -> Vec<Equation> {
        let lines = &[
            "190: 10 19",
            "3267: 81 40 27",
            "83: 17 5",
            "156: 15 6",
            "7290: 6 8 6 15",
            "161011: 16 10 13",
            "192: 17 8 14",
            "21037: 9 7 18 13",
            "292: 11 6 16 20",
        ];
        lines
            .iter()
            .map(|s| Equation::from_str(s))
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn equations_are_validated_successfully() {
        let es = get_test_equations();

        assert!(es[0].is_valid());
        assert!(es[1].is_valid());
        assert!(!es[2].is_valid());
        assert!(es[3].is_valid());
        assert!(es[4].is_valid());
        assert!(!es[5].is_valid());
        assert!(es[6].is_valid());
        assert!(!es[7].is_valid());
        assert!(es[8].is_valid());
    }

    #[test]
    fn valid_equations_produce_expected_total() {
        let es = get_test_equations().into_iter().filter(Equation::is_valid);

        assert_eq!(sum_results(es), 11387);
    }

    #[test]
    fn edge_case_equations_are_validated() {
        let e = Equation {
            result: 349_510,
            values: vec![3, 587, 66, 1, 126, 3, 451],
        };
        assert!(e.is_valid());

        let e = Equation {
            result: 7,
            values: vec![6],
        };
        assert!(!e.is_valid());
    }

    #[test]
    fn disjoining_numbers_succeeds_for_valid_numbers() {
        assert_eq!(disjoin(1234, 34), Some(12));
        assert_eq!(disjoin(12345, 345), Some(12));
        assert!(disjoin(1234, 35).is_none());
        assert!(disjoin(34, 1234).is_none());
        assert!(disjoin(0, 0).is_none());
    }
}
