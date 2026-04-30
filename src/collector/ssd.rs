use super::{Collector, Snapshot};
use anyhow::{Context, Result};
use std::process::Command;

pub struct SsdCollector;

impl Collector for SsdCollector {
    fn collect(&self) -> Result<Snapshot> {
        let output = Command::new("smartctl")
            .args(["-j", "-a", "-d", "nvme", "/dev/disk0"])
            .output()
            .context("failed to run smartctl — is smartmontools installed? (brew install smartmontools)")?;

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).context("failed to parse smartctl JSON")?;

        let health = &json["nvme_smart_health_information_log"];
        let model = json["model_name"].as_str().map(String::from);

        Ok(Snapshot {
            power_on_hours: health["power_on_hours"].as_i64(),
            power_cycles: health["power_cycles"].as_i64(),
            data_units_read: health["data_units_read"].as_i64(),
            data_units_written: health["data_units_written"].as_i64(),
            percentage_used: health["percentage_used"].as_i64(),
            available_spare: health["available_spare"].as_i64(),
            unsafe_shutdowns: health["unsafe_shutdowns"].as_i64(),
            integrity_errors: health["media_errors"].as_i64(),
            ssd_temp_c: health["temperature"].as_i64(),
            ssd_model: model,
            ..Default::default()
        })
    }
}
