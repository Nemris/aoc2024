#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

/// Pattern to extract operands from `mul(m,n)` instructions.
static MUL_OPERANDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"mul\(([0-9]{1,3}),([0-9]{1,3})\)").expect("pattern creation should succeed")
});

/// Tries to extract the operands from all `mul(m,n)` instructions.
fn extract_mul_operands(hay: &str) -> Result<Vec<(u32, u32)>, ParseIntError> {
    let mut operands = vec![];
    for (_, [m, n]) in MUL_OPERANDS_RE.captures_iter(hay).map(|c| c.extract()) {
        operands.push((m.parse()?, n.parse()?));
    }

    Ok(operands)
}

/// Multiplies pairs of operands and sums the results.
fn compute_total(ops: &[(u32, u32)]) -> u32 {
    ops.iter().map(|(m, n)| m * n).sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let reader = BufReader::new(File::open(dataset)?);

    let mut operands: Vec<(u32, u32)> = vec![];
    for line in reader.lines() {
        operands.extend(extract_mul_operands(&line?)?.iter());
    }

    let total = compute_total(&operands);
    println!("Total: {total}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HAY: &str = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";

    #[test]
    fn pattern_can_extract_mul_operands() {
        assert_eq!(
            &extract_mul_operands(HAY).unwrap(),
            &[(2, 4), (5, 5), (11, 8), (8, 5)]
        );
    }

    #[test]
    fn operands_total_computes_successfully() {
        assert_eq!(compute_total(&[(2, 4), (5, 5), (11, 8), (8, 5)]), 161);
    }
}
