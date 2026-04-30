use crate::db::Row;
use anyhow::Result;

pub fn export_json(rows: &[Row]) -> Result<()> {
    let json = serde_json::to_string_pretty(rows)?;
    println!("{}", json);
    Ok(())
}
