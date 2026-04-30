use crate::db::Row;
use anyhow::Result;

pub fn export_json(rows: &[Row]) -> Result<()> {
    let json = serde_json::to_string_pretty(rows)?;
    println!("{}", json);
    Ok(())
}

pub fn export_csv(rows: &[Row]) -> Result<()> {
    println!("ts,power_on_hours,power_cycles,data_units_read,data_units_written,percentage_used,available_spare,unsafe_shutdowns,integrity_errors,ssd_temp_c,cycle_count,max_capacity_pct,design_capacity,condition");
    for r in rows {
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            r.ts,
            opt(r.power_on_hours),
            opt(r.power_cycles),
            opt(r.data_units_read),
            opt(r.data_units_written),
            opt(r.percentage_used),
            opt(r.available_spare),
            opt(r.unsafe_shutdowns),
            opt(r.integrity_errors),
            opt(r.ssd_temp_c),
            opt(r.cycle_count),
            opt(r.max_capacity_pct),
            opt(r.design_capacity),
            r.condition.as_deref().unwrap_or(""),
        );
    }
    Ok(())
}

fn opt(v: Option<i64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_default()
}
