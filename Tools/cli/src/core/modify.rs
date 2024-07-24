use std::path::Path;

use anyhow::Result;

pub enum Modification {
    Stackable(bool),
}

pub fn modify(dir: &Path, item_id: u32, modification: &Modification) -> Result<()> {
    let item_path = dir.join(format!("{item_id}.json"));

    Ok(())
}
