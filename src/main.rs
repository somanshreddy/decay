mod cli;
mod collector;
mod db;
mod display;
mod export;
mod predict;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, ExportFormat};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = db::open()?;

    match cli.command {
        Some(Command::Snapshot) => {
            let snapshot = collector::collect_all()?;
            db::insert(&conn, &snapshot)?;
            println!("  ✅ Snapshot saved.");
            println!();
            let rows = db::recent(&conn, 30)?;
            display::summary::print_summary(&snapshot, &rows);
        }
        Some(Command::History { count }) => {
            let rows = db::recent(&conn, count)?;
            display::history::print_history(&rows);
        }
        Some(Command::Predict) => {
            let rows = db::all(&conn)?;
            let predictions = predict::predict(&rows);
            if predictions.is_empty() {
                println!("  Need more snapshots for predictions. Run `decay snapshot` daily.");
            } else {
                println!();
                for p in &predictions {
                    println!("  🛞 {}: {}", p.label, p.message);
                }
                println!();
            }
        }
        Some(Command::Export { format }) => match format {
            ExportFormat::Json => {
                let rows = db::all(&conn)?;
                export::export_json(&rows)?;
            }
        },
        None => {
            let snapshot = collector::collect_all()?;
            let rows = db::recent(&conn, 30)?;
            display::summary::print_summary(&snapshot, &rows);
        }
    }

    Ok(())
}
