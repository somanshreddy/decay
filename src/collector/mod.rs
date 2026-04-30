pub mod battery;
pub mod benchmark;
pub mod ssd;
pub mod temperature;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    // SSD
    pub power_on_hours: Option<i64>,
    pub power_cycles: Option<i64>,
    pub data_units_read: Option<i64>,
    pub data_units_written: Option<i64>,
    pub percentage_used: Option<i64>,
    pub available_spare: Option<i64>,
    pub unsafe_shutdowns: Option<i64>,
    pub integrity_errors: Option<i64>,
    pub ssd_temp_c: Option<i64>,
    pub ssd_model: Option<String>,
    // Battery
    pub cycle_count: Option<i64>,
    pub max_capacity_pct: Option<i64>,
    pub design_capacity: Option<i64>,
    pub condition: Option<String>,
    // CPU temperature
    pub cpu_temp_c: Option<i64>,
    // Disk I/O benchmark (MB/s)
    pub disk_read_mbs: Option<i64>,
    pub disk_write_mbs: Option<i64>,
}

pub trait Collector {
    fn collect(&self) -> Result<Snapshot>;
}

pub fn collect_all() -> Result<Snapshot> {
    let ssd = ssd::SsdCollector::new().collect().unwrap_or_default();
    let bat = battery::collect_battery().unwrap_or_default();
    let temp = temperature::collect_temperature().unwrap_or_default();
    let bench = benchmark::collect_benchmark().unwrap_or_default();

    Ok(Snapshot {
        power_on_hours: ssd.power_on_hours,
        power_cycles: ssd.power_cycles,
        data_units_read: ssd.data_units_read,
        data_units_written: ssd.data_units_written,
        percentage_used: ssd.percentage_used,
        available_spare: ssd.available_spare,
        unsafe_shutdowns: ssd.unsafe_shutdowns,
        integrity_errors: ssd.integrity_errors,
        ssd_temp_c: ssd.ssd_temp_c,
        ssd_model: ssd.ssd_model,
        cycle_count: bat.cycle_count,
        max_capacity_pct: bat.max_capacity_pct,
        design_capacity: bat.design_capacity,
        condition: bat.condition,
        cpu_temp_c: temp.cpu_temp_c,
        disk_read_mbs: bench.disk_read_mbs,
        disk_write_mbs: bench.disk_write_mbs,
    })
}
