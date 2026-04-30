use crate::db::Row;

pub struct Prediction {
    pub label: &'static str,
    pub message: String,
}

pub fn predict(rows: &[Row]) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    if let Some(msg) = predict_ssd_wear(rows) {
        predictions.push(Prediction {
            label: "SSD",
            message: msg,
        });
    }

    if let Some(msg) = predict_battery(rows) {
        predictions.push(Prediction {
            label: "Battery",
            message: msg,
        });
    }

    predictions
}

fn predict_ssd_wear(rows: &[Row]) -> Option<String> {
    let points: Vec<(f64, f64)> = rows
        .iter()
        .filter_map(|r| {
            let ts = parse_ts(&r.ts)?;
            let pct = r.percentage_used? as f64;
            Some((ts, pct))
        })
        .collect();

    if points.len() < 2 {
        return None;
    }

    let (slope, _intercept) = ols(&points);

    if slope <= 0.0 {
        return Some("SSD wear is flat — cruising with no visible degradation".into());
    }

    let current = points.last().unwrap().1;
    let hours_to_100 = (100.0 - current) / slope;
    let years = hours_to_100 / (24.0 * 365.25);

    if years > 50.0 {
        Some("SSD has mass miles left — wear rate is near zero".into())
    } else if years > 10.0 {
        Some(format!(
            "SSD has mass miles left — 100% wear in ~{:.0} years at this pace",
            years
        ))
    } else if years > 3.0 {
        Some(format!(
            "SSD holding up — 100% wear in ~{:.1} years at this pace",
            years
        ))
    } else if years > 1.0 {
        Some(format!(
            "SSD wearing down — 100% wear in ~{:.1} years at this pace",
            years
        ))
    } else {
        let months = years * 12.0;
        Some(format!(
            "⚠️  SSD wearing fast — 100% wear in ~{:.0} months at this pace",
            months
        ))
    }
}

fn predict_battery(rows: &[Row]) -> Option<String> {
    let points: Vec<(f64, f64)> = rows
        .iter()
        .filter_map(|r| {
            let ts = parse_ts(&r.ts)?;
            let pct = r.max_capacity_pct? as f64;
            Some((ts, pct))
        })
        .collect();

    if points.len() < 2 {
        return None;
    }

    let (slope, _intercept) = ols(&points);

    if slope >= 0.0 {
        return Some("Battery health is steady — no degradation trend yet".into());
    }

    let current = points.last().unwrap().1;
    let hours_to_80 = (current - 80.0) / slope.abs();
    let years = hours_to_80 / (24.0 * 365.25);

    if current <= 80.0 {
        Some("Battery below 80% — Apple considers this service territory".into())
    } else if years > 10.0 {
        Some("Battery cruising — 80% health is years away at this rate".into())
    } else if years > 3.0 {
        Some(format!(
            "Battery cruising — 80% health in ~{:.1} years at current rate",
            years
        ))
    } else if years > 1.0 {
        Some(format!(
            "Battery degrading — 80% health in ~{:.1} years at current rate",
            years
        ))
    } else {
        let months = years * 12.0;
        Some(format!(
            "⚠️  Battery dropping — 80% health in ~{:.0} months at current rate",
            months
        ))
    }
}

fn parse_ts(ts: &str) -> Option<f64> {
    // Parse "2026-04-30T02:49:13" into hours since epoch (approximate)
    let parts: Vec<&str> = ts.split('T').collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<i64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    let time_parts: Vec<i64> = parts[1].split(':').filter_map(|s| s.parse().ok()).collect();
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }
    let days = date_parts[0] * 365 + date_parts[1] * 30 + date_parts[2];
    let hours = days as f64 * 24.0 + time_parts[0] as f64 + time_parts[1] as f64 / 60.0;
    Some(hours)
}

/// Ordinary least squares: returns (slope, intercept) for y = slope*x + intercept
fn ols(points: &[(f64, f64)]) -> (f64, f64) {
    let n = points.len() as f64;
    let sum_x: f64 = points.iter().map(|p| p.0).sum();
    let sum_y: f64 = points.iter().map(|p| p.1).sum();
    let sum_xy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let sum_x2: f64 = points.iter().map(|p| p.0 * p.0).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return (0.0, sum_y / n);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;
    (slope, intercept)
}
