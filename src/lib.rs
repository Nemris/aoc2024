#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};

/// Builds the path to a dataset paired to a specific solution binary.
///
/// # Panics
///
/// This function may panic if `source_path` does not end with a filename.
#[must_use]
pub fn get_dataset(source_path: &Path, dataset_name: &str) -> PathBuf {
    let source_name = {
        let n = source_path
            .file_name()
            .expect("source file's name should exist")
            .to_str()
            .expect("converting filename back to str should succeed");
        n.strip_suffix(".rs").unwrap_or(n)
    };

    [
        env!("CARGO_MANIFEST_DIR"),
        "resources",
        source_name,
        dataset_name,
    ]
    .iter()
    .collect()
}
