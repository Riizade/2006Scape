use std::path::Path;

use super::item_definition::{self, ItemDefinition};
use anyhow::{Ok, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

/// uses the given string as a regular expression and finds all item definitions with matching names
fn search(dir: &Path, expression: &str) -> Result<Vec<ItemDefinition>> {
    let all_items = item_definition::load_all(dir)?;

    let regex = Regex::new(expression)?;

    let result = all_items
        .par_iter()
        .filter(|item| regex.is_match(&item.name))
        .map(|item| item.clone())
        .collect();

    Ok(result)
}
