use super::{Collector, Snapshot};
use anyhow::Result;
use std::process::Command;

pub struct BatteryCollector;

impl Collector for BatteryCollector {
    fn collect(&self) -> Result<Snapshot> {
        let cycle_count = read_ioreg_i64("CycleCount");
        let max_capacity_pct = read_ioreg_i64("MaxCapacity");
        let design_capacity = read_ioreg_i64("DesignCapacity");
        let condition = read_condition();

        Ok(Snapshot {
            cycle_count,
            max_capacity_pct,
            design_capacity,
            condition,
            ..Default::default()
        })
    }
}

fn read_ioreg_i64(key: &str) -> Option<i64> {
    let output = Command::new("ioreg")
        .args(["-rn", "AppleSmartBattery", "-w0"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Match top-level keys like `"CycleCount" = 42` but not nested ones inside BatteryData
    for line in stdout.lines() {
        let trimmed = line.trim();
        // Skip lines that are deeply nested (inside dicts like BatteryData)
        // Top-level ioreg keys have moderate indentation (6-8 spaces)
        let indent = line.len() - line.trim_start().len();
        if indent > 12 {
            continue;
        }
        let pattern = format!("\"{}\"", key);
        if trimmed.starts_with(&pattern) {
            if let Some(val) = trimmed.split('=').nth(1) {
                return val.trim().parse().ok();
            }
        }
    }
    None
}

fn read_condition() -> Option<String> {
    let output = Command::new("system_profiler")
        .args(["SPPowerDataType"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("Condition") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}
