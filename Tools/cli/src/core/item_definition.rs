use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// a struct only intended to read certain fields, do not write using this struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemDefinition {
    pub name: String,
    pub id: u32,
    pub stackable: bool,
    pub value: i32,
}

pub fn load_all(dir: &Path) -> Result<Vec<ItemDefinition>> {
    let mut items: Vec<_> = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry?.path();
        let contents = std::fs::read_to_string(path)?;
        let item: ItemDefinition = serde_json::from_str(&contents)?;
        items.push(item);
    }

    Ok(items)
}
