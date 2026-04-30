use crate::db::Row;
use owo_colors::OwoColorize;

pub fn print_history(rows: &[Row]) {
    if rows.is_empty() {
        println!("  No snapshots yet. Run `decay snapshot` to take one.");
        return;
    }

    println!();
    println!(
        "  {:<20} {:>6} {:>6} {:>10} {:>10} {:>5} {:>5}",
        "Timestamp".bold(),
        "Wear%".bold(),
        "Spare".bold(),
        "Written".bold(),
        "Read".bold(),
        "Batt%".bold(),
        "Cyc".bold(),
    );
    println!(
        "  {:<20} {:>6} {:>6} {:>10} {:>10} {:>5} {:>5}",
        "─".repeat(19),
        "─".repeat(5),
        "─".repeat(5),
        "─".repeat(9),
        "─".repeat(9),
        "─".repeat(5),
        "─".repeat(5),
    );

    for row in rows {
        let ts = &row.ts[..19.min(row.ts.len())];
        let wear = row
            .percentage_used
            .map(|v| format!("{}%", v))
            .unwrap_or_else(|| "—".into());
        let spare = row
            .available_spare
            .map(|v| format!("{}%", v))
            .unwrap_or_else(|| "—".into());
        let written = row
            .data_units_written
            .map(|v| format_short_units(v))
            .unwrap_or_else(|| "—".into());
        let read = row
            .data_units_read
            .map(|v| format_short_units(v))
            .unwrap_or_else(|| "—".into());
        let batt = row
            .max_capacity_pct
            .map(|v| format!("{}%", v))
            .unwrap_or_else(|| "—".into());
        let cyc = row
            .cycle_count
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".into());

        println!(
            "  {:<20} {:>6} {:>6} {:>10} {:>10} {:>5} {:>5}",
            ts.dimmed(),
            wear,
            spare,
            written,
            read,
            batt,
            cyc,
        );
    }
    println!();
}

fn format_short_units(data_units: i64) -> String {
    let bytes = data_units as f64 * 512.0 * 1000.0;
    if bytes >= 1e12 {
        format!("{:.1} TB", bytes / 1e12)
    } else if bytes >= 1e9 {
        format!("{:.1} GB", bytes / 1e9)
    } else {
        format!("{:.0} MB", bytes / 1e6)
    }
}
