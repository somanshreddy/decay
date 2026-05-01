use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "decay",
    about = "🚗 Are you riding your machine until the wheels fall off?",
    long_about = "Track SSD wear & battery decay over time.\nRun `decay snapshot` daily, then `decay` to see how your machine is holding up."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Take a snapshot of current SSD and battery health
    Snapshot,
    /// Show snapshot history
    History {
        /// Number of recent snapshots to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
    /// Predict when SSD/battery hit critical thresholds
    Predict,
    /// Interactive TUI chart of health trends
    Chart,
    /// Export all snapshots
    Export {
        /// Output format
        #[arg(long, default_value = "json")]
        format: ExportFormat,
    },
    /// Set up daily automatic snapshots (macOS launchd)
    Install,
    /// Remove the daily snapshot schedule
    Uninstall,
}

#[derive(Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
}
