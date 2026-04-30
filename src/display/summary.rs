use crate::collector::Snapshot;
use crate::db::Row;
use crate::predict;
use owo_colors::OwoColorize;

const SPARK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn sparkline(values: &[i64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let min = *values.iter().min().unwrap() as f64;
    let max = *values.iter().max().unwrap() as f64;
    let range = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };
    values
        .iter()
        .map(|&v| {
            let idx = (((v as f64 - min) / range) * 7.0) as usize;
            SPARK_CHARS[idx.min(7)]
        })
        .collect()
}

fn format_data_units(units: i64) -> String {
    let bytes = units as f64 * 512.0 * 1000.0;
    if bytes >= 1e12 {
        format!("{:.2} TB", bytes / 1e12)
    } else if bytes >= 1e9 {
        format!("{:.2} GB", bytes / 1e9)
    } else {
        format!("{:.0} MB", bytes / 1e6)
    }
}

fn commafy(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

pub fn print_summary(current: &Snapshot, history: &[Row]) {
    println!();
    println!(
        "  {}",
        "🚗 decay — how many miles left?".bold()
    );
    println!();

    // SSD section
    let model = current
        .ssd_model
        .as_deref()
        .unwrap_or("Unknown SSD");
    println!("  {} {}", "SSD".bold(), model.dimmed());

    if let Some(pct) = current.percentage_used {
        let wear_spark = sparkline(
            &history
                .iter()
                .filter_map(|r| r.percentage_used)
                .collect::<Vec<_>>(),
        );
        let spare = current.available_spare.unwrap_or(0);
        let temp = current
            .ssd_temp_c
            .map(|t| format!("{}°C", t))
            .unwrap_or_else(|| "?".into());
        println!(
            "    Wear: {:<3} {}  Spare: {}%  Temp: {}",
            format!("{}%", pct).yellow(),
            wear_spark.dimmed(),
            spare,
            temp,
        );
    }

    if let (Some(written), Some(read)) = (current.data_units_written, current.data_units_read) {
        let cycles = current.power_cycles.unwrap_or(0);
        println!(
            "    Written: {}  Read: {}  Cycles: {}",
            format_data_units(written).cyan(),
            format_data_units(read).cyan(),
            commafy(cycles).cyan(),
        );
    }

    if let Some(unsafe_sd) = current.unsafe_shutdowns {
        let errs = current.integrity_errors.unwrap_or(0);
        println!(
            "    Unsafe shutdowns: {}  Integrity errors: {}",
            if unsafe_sd > 0 {
                commafy(unsafe_sd).yellow().to_string()
            } else {
                commafy(unsafe_sd).green().to_string()
            },
            if errs > 0 {
                commafy(errs).red().to_string()
            } else {
                commafy(errs).green().to_string()
            },
        );
    }

    println!();

    // Battery section
    println!("  {}", "Battery".bold());
    if let Some(health) = current.max_capacity_pct {
        let health_spark = sparkline(
            &history
                .iter()
                .filter_map(|r| r.max_capacity_pct)
                .collect::<Vec<_>>(),
        );
        let cycles = current.cycle_count.unwrap_or(0);
        let cond = current
            .condition
            .as_deref()
            .unwrap_or("Unknown");
        let health_color = if health >= 80 {
            format!("{}%", health).green().to_string()
        } else if health >= 50 {
            format!("{}%", health).yellow().to_string()
        } else {
            format!("{}%", health).red().to_string()
        };
        println!(
            "    Health: {:<5} {}  Cycles: {} / 1000  Condition: {}",
            health_color,
            health_spark.dimmed(),
            commafy(cycles),
            cond,
        );
    }

    if let Some(design) = current.design_capacity {
        println!("    Design capacity: {} mAh", commafy(design));
    }

    // System health section
    let has_temp = current.cpu_temp_c.is_some();
    let has_bench = current.disk_read_mbs.is_some() || current.disk_write_mbs.is_some();
    if has_temp || has_bench {
        println!("  {}", "System".bold());
        if let Some(temp) = current.cpu_temp_c {
            let temp_spark = sparkline(
                &history
                    .iter()
                    .filter_map(|r| r.cpu_temp_c)
                    .collect::<Vec<_>>(),
            );
            let temp_color = if temp >= 90 {
                format!("{}°C", temp).red().to_string()
            } else if temp >= 70 {
                format!("{}°C", temp).yellow().to_string()
            } else {
                format!("{}°C", temp).green().to_string()
            };
            println!("    CPU temp: {}  {}", temp_color, temp_spark.dimmed());
        }
        if let (Some(read), Some(write)) = (current.disk_read_mbs, current.disk_write_mbs) {
            let read_spark = sparkline(
                &history
                    .iter()
                    .filter_map(|r| r.disk_read_mbs)
                    .collect::<Vec<_>>(),
            );
            println!(
                "    Disk I/O: {} read  {} write  {}",
                format!("{} MB/s", read).cyan(),
                format!("{} MB/s", write).cyan(),
                read_spark.dimmed(),
            );
        }
        println!();
    }

    // Mileage estimates
    let predictions = predict::predict(history);
    if predictions.is_empty() {
        let snap_count = history.len();
        let needed = if snap_count < 2 { 2 - snap_count } else { 0 };
        if needed > 0 {
            println!(
                "  {}",
                format!(
                    "🛞 Run `decay snapshot` daily — need {} more readings for mileage estimates",
                    needed
                )
                .dimmed()
            );
        }
    } else {
        for p in &predictions {
            println!("  🛞 {}: {}", p.label, p.message);
        }
    }

    println!();
}
