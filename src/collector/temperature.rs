use super::Snapshot;
use anyhow::Result;

pub fn collect_temperature() -> Result<Snapshot> {
    #[cfg(target_os = "macos")]
    let cpu_temp_c = collect_macos();

    #[cfg(target_os = "linux")]
    let cpu_temp_c = collect_linux();

    #[cfg(target_os = "windows")]
    let cpu_temp_c = collect_windows();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let cpu_temp_c: Option<i64> = None;

    Ok(Snapshot {
        cpu_temp_c,
        ..Default::default()
    })
}

#[cfg(target_os = "macos")]
fn collect_macos() -> Option<i64> {
    // Try reading CPU die temperature from powermetrics (needs sudo)
    // Fall back to SMC temperature via ioreg
    let output = std::process::Command::new("ioreg")
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
        if trimmed.starts_with("\"Temperature\"")
            && let Some(val) = trimmed.split('=').nth(1)
        {
            let raw: i64 = val.trim().parse().ok()?;
            return Some((raw / 10) - 273);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn collect_linux() -> Option<i64> {
    // Read from thermal zones — find the CPU one
    let thermal = std::path::Path::new("/sys/class/thermal");
    for entry in std::fs::read_dir(thermal).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let type_file = path.join("type");
        if let Ok(t) = std::fs::read_to_string(&type_file) {
            let t = t.trim();
            // Common CPU thermal zone names
            if t.contains("cpu") || t.contains("x86_pkg") || t.contains("coretemp")
                || t.contains("acpitz") || t.contains("k10temp") || t.contains("zenpower")
            {
                if let Ok(temp_str) = std::fs::read_to_string(path.join("temp")) {
                    if let Ok(millideg) = temp_str.trim().parse::<i64>() {
                        return Some(millideg / 1000);
                    }
                }
            }
        }
    }
    // Fall back to first thermal zone
    let temp_path = thermal.join("thermal_zone0").join("temp");
    if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
        if let Ok(millideg) = temp_str.trim().parse::<i64>() {
            return Some(millideg / 1000);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn collect_windows() -> Option<i64> {
    let output = std::process::Command::new("wmic")
        .args(["path", "MSAcpi_ThermalZoneTemperature", "get", "CurrentTemperature", "/format:list"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(val) = line.trim().strip_prefix("CurrentTemperature=") {
            // WMI returns deciKelvin
            let raw: i64 = val.parse().ok()?;
            return Some((raw / 10) - 273);
        }
    }
    None
}
