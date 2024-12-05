#![warn(clippy::pedantic)]

use std::error::Error;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

/// Pattern to extract operands from `mul(m,n)` instructions.
static MUL_OPERANDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"mul\(([0-9]{1,3}),([0-9]{1,3})\)").expect("pattern creation should succeed")
});

/// Pattern to identify an ignored program region.
///
/// Regions to be ignored start from `don't()` and continue until the following `do()` or the end
/// of the input, whichever comes first.
static IGNORED_REGION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)don't\(\)(.*?do\(\)|.*$)").expect("pattern creation should succeed")
});

/// Tries to extract the operands from all `mul(m,n)` instructions.
fn extract_mul_operands(hay: &str) -> Result<Vec<(u32, u32)>, ParseIntError> {
    let mut operands = vec![];
    for (_, [m, n]) in MUL_OPERANDS_RE.captures_iter(hay).map(|c| c.extract()) {
        operands.push((m.parse()?, n.parse()?));
    }

    Ok(operands)
}

/// Tries to extract the operands from all enabled `mul(m,n)` instructions.
fn extract_enabled_mul_operands(hay: &str) -> Result<Vec<(u32, u32)>, ParseIntError> {
    extract_mul_operands(&IGNORED_REGION_RE.replace_all(hay, ""))
}

/// Multiplies pairs of operands and sums the results.
fn compute_total(ops: &[(u32, u32)]) -> u32 {
    ops.iter().map(|(m, n)| m * n).sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let data = fs::read_to_string(dataset)?;

    let operands = extract_mul_operands(&data)?;
    let enabled_operands = extract_enabled_mul_operands(&data)?;

    println!("Total (all muls):     {}", compute_total(&operands));
    println!("Total (enabled muls): {}", compute_total(&enabled_operands));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HAY: &str = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
    const HAY_WITH_DISABLED_REGIONS: &str =
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))do()";

    #[test]
    fn pattern_can_extract_mul_operands() {
        assert_eq!(
            &extract_mul_operands(HAY).unwrap(),
            &[(2, 4), (5, 5), (11, 8), (8, 5)]
        );
    }

    #[test]
    fn pattern_can_extract_operands_from_enabled_mul() {
        assert_eq!(
            &extract_enabled_mul_operands(HAY_WITH_DISABLED_REGIONS).unwrap(),
            &[(2, 4), (8, 5)]
        );
    }

    #[test]
    fn operands_total_computes_successfully() {
        assert_eq!(compute_total(&[(2, 4), (5, 5), (11, 8), (8, 5)]), 161);
    }
}
