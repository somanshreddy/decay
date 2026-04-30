use super::{Collector, Snapshot};
use anyhow::{Context, Result};
use std::process::Command;

pub struct SsdCollector {
    device: String,
    extra_args: Vec<String>,
}

impl SsdCollector {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        return Self {
            device: "/dev/disk0".into(),
            extra_args: vec!["-d".into(), "nvme".into()],
        };

        #[cfg(target_os = "linux")]
        return Self::detect_linux();

        #[cfg(target_os = "windows")]
        return Self {
            device: "/dev/sda".into(),
            extra_args: vec![],
        };

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        Self {
            device: "/dev/sda".into(),
            extra_args: vec![],
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux() -> Self {
        // Try NVMe first, fall back to SATA
        for dev in &["/dev/nvme0n1", "/dev/sda"] {
            if std::path::Path::new(dev).exists() {
                return Self {
                    device: dev.to_string(),
                    extra_args: vec![],
                };
            }
        }
        Self {
            device: "/dev/sda".into(),
            extra_args: vec![],
        }
    }
}

impl Collector for SsdCollector {
    fn collect(&self) -> Result<Snapshot> {
        let mut args = vec!["-j".to_string(), "-a".to_string()];
        args.extend(self.extra_args.clone());
        args.push(self.device.clone());

        let output = Command::new("smartctl")
            .args(&args)
            .output()
            .context("failed to run smartctl — is smartmontools installed?")?;

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).context("failed to parse smartctl JSON")?;

        let health = &json["nvme_smart_health_information_log"];
        let model = json["model_name"].as_str().map(String::from);

        // NVMe uses nvme_smart_health_information_log, SATA uses ata_smart_attributes
        if health.is_object() {
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
        } else {
            // SATA fallback — parse from ata_smart_attributes
            let attrs = &json["ata_smart_attributes"]["table"];
            Ok(Snapshot {
                power_on_hours: find_attr(attrs, 9),
                power_cycles: find_attr(attrs, 12),
                percentage_used: find_attr(attrs, 177)
                    .or_else(|| find_attr(attrs, 231)),
                ssd_temp_c: find_attr(attrs, 194),
                ssd_model: model,
                ..Default::default()
            })
        }
    }
}

fn find_attr(table: &serde_json::Value, id: i64) -> Option<i64> {
    table.as_array()?.iter().find_map(|attr| {
        if attr["id"].as_i64()? == id {
            attr["raw"]["value"].as_i64()
        } else {
            None
        }
    })
}
