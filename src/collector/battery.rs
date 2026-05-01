use super::Snapshot;
use anyhow::Result;
use std::process::Command;

pub fn collect_battery() -> Result<Snapshot> {
    #[cfg(target_os = "macos")]
    return collect_macos();

    #[cfg(target_os = "linux")]
    return collect_linux();

    #[cfg(target_os = "windows")]
    return collect_windows();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    Ok(Snapshot::default())
}

#[cfg(target_os = "macos")]
fn collect_macos() -> Result<Snapshot> {
    let cycle_count = read_ioreg_i64("CycleCount");
    let max_capacity_pct = read_ioreg_i64("MaxCapacity");
    let design_capacity = read_ioreg_i64("DesignCapacity");
    let condition = read_condition_macos();

    Ok(Snapshot {
        cycle_count,
        max_capacity_pct,
        design_capacity,
        condition,
        ..Default::default()
    })
}

#[cfg(target_os = "macos")]
fn read_ioreg_i64(key: &str) -> Option<i64> {
    let output = Command::new("ioreg")
        .args(["-rn", "AppleSmartBattery", "-w0"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        if indent > 12 {
            continue;
        }
        let pattern = format!("\"{}\"", key);
        if trimmed.starts_with(&pattern)
            && let Some(val) = trimmed.split('=').nth(1)
        {
            return val.trim().parse().ok();
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn read_condition_macos() -> Option<String> {
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

#[cfg(target_os = "linux")]
fn collect_linux() -> Result<Snapshot> {
    let base = find_battery_path()?;

    let cycle_count = read_sysfs_i64(&base.join("cycle_count"));
    let design_capacity = read_sysfs_i64(&base.join("charge_full_design"))
        .or_else(|| read_sysfs_i64(&base.join("energy_full_design")));
    let current_capacity = read_sysfs_i64(&base.join("charge_full"))
        .or_else(|| read_sysfs_i64(&base.join("energy_full")));

    let max_capacity_pct = match (current_capacity, design_capacity) {
        (Some(cur), Some(des)) if des > 0 => Some((cur * 100) / des),
        _ => None,
    };

    let status = read_sysfs_string(&base.join("status"));
    let condition = status.map(|s| match s.trim() {
        "Full" | "Charging" | "Not charging" => "Normal".to_string(),
        "Discharging" => "Normal".to_string(),
        other => other.to_string(),
    });

    // Convert µAh to mAh if charge_full_design is in µAh
    let design_mah = design_capacity.map(|d| if d > 100_000 { d / 1000 } else { d });

    Ok(Snapshot {
        cycle_count,
        max_capacity_pct,
        design_capacity: design_mah,
        condition,
        ..Default::default()
    })
}

#[cfg(target_os = "linux")]
fn find_battery_path() -> Result<std::path::PathBuf> {
    let power_supply = std::path::Path::new("/sys/class/power_supply");
    for entry in std::fs::read_dir(power_supply)? {
        let entry = entry?;
        let path = entry.path();
        let type_file = path.join("type");
        if let Ok(t) = std::fs::read_to_string(&type_file) {
            if t.trim() == "Battery" {
                return Ok(path);
            }
        }
    }
    anyhow::bail!("no battery found in /sys/class/power_supply/")
}

#[cfg(target_os = "linux")]
fn read_sysfs_i64(path: &std::path::Path) -> Option<i64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

#[cfg(target_os = "linux")]
fn read_sysfs_string(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

#[cfg(target_os = "windows")]
fn collect_windows() -> Result<Snapshot> {
    let output = Command::new("wmic")
        .args(["path", "Win32_Battery", "get",
               "DesignCapacity,FullChargeCapacity,EstimatedChargeRemaining,Status",
               "/format:list"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut design_capacity = None;
    let mut full_charge = None;
    let mut condition = None;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("DesignCapacity=") {
            design_capacity = val.parse().ok();
        } else if let Some(val) = line.strip_prefix("FullChargeCapacity=") {
            full_charge = val.parse::<i64>().ok();
        } else if let Some(val) = line.strip_prefix("Status=") {
            condition = Some(val.to_string());
        }
    }

    let max_capacity_pct = match (full_charge, design_capacity) {
        (Some(cur), Some(des)) if des > 0 => Some((cur * 100) / des),
        _ => None,
    };

    // Windows doesn't expose cycle count via WMI — would need powercfg /batteryreport
    Ok(Snapshot {
        cycle_count: None,
        max_capacity_pct,
        design_capacity,
        condition,
        ..Default::default()
    })
}
