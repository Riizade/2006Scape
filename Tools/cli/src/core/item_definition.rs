use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub ground_actions: Option<[String; 5]>,
    pub inventory_actions: Option<[String; 5]>,
    pub members: bool,
    pub note_graphic_id: Option<i32>,
    pub note_info_id: Option<i32>,
    pub team: i32,
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
