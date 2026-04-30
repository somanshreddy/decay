mod cli;
mod collector;
mod db;
mod display;
mod export;
mod predict;
mod scheduler;

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
        Some(Command::Chart) => {
            let rows = db::all(&conn)?;
            display::chart::run(&rows)?;
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
        Some(Command::Export { format }) => {
            let rows = db::all(&conn)?;
            match format {
                ExportFormat::Json => export::export_json(&rows)?,
                ExportFormat::Csv => export::export_csv(&rows)?,
            }
        }
        Some(Command::Install) => {
            scheduler::install()?;
        }
        Some(Command::Uninstall) => {
            scheduler::uninstall()?;
        }
        None => {
            let snapshot = collector::collect_all()?;
            let rows = db::recent(&conn, 30)?;
            display::summary::print_summary(&snapshot, &rows);
        }
    }

    Ok(())
}
