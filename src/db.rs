use crate::collector::Snapshot;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::PathBuf;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS snapshots (
  id                 INTEGER PRIMARY KEY,
  ts                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now','localtime')),
  power_on_hours     INTEGER,
  power_cycles       INTEGER,
  data_units_read    INTEGER,
  data_units_written INTEGER,
  percentage_used    INTEGER,
  available_spare    INTEGER,
  unsafe_shutdowns   INTEGER,
  integrity_errors   INTEGER,
  ssd_temp_c         INTEGER,
  cycle_count        INTEGER,
  max_capacity_pct   INTEGER,
  design_capacity    INTEGER,
  condition          TEXT
);
CREATE INDEX IF NOT EXISTS idx_ts ON snapshots(ts);
";

fn db_path() -> Result<PathBuf> {
    let dir = dirs();
    std::fs::create_dir_all(&dir).context("failed to create data directory")?;
    Ok(dir.join("decay.db"))
}

fn dirs() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("decay")
}

pub fn open() -> Result<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path).context("failed to open database")?;
    conn.execute_batch(SCHEMA)
        .context("failed to initialize schema")?;
    Ok(conn)
}

pub fn insert(conn: &Connection, s: &Snapshot) -> Result<()> {
    conn.execute(
        "INSERT INTO snapshots (
            power_on_hours, power_cycles, data_units_read, data_units_written,
            percentage_used, available_spare, unsafe_shutdowns, integrity_errors,
            ssd_temp_c, cycle_count, max_capacity_pct, design_capacity, condition
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![
            s.power_on_hours,
            s.power_cycles,
            s.data_units_read,
            s.data_units_written,
            s.percentage_used,
            s.available_spare,
            s.unsafe_shutdowns,
            s.integrity_errors,
            s.ssd_temp_c,
            s.cycle_count,
            s.max_capacity_pct,
            s.design_capacity,
            s.condition,
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub ts: String,
    pub power_on_hours: Option<i64>,
    pub power_cycles: Option<i64>,
    pub data_units_read: Option<i64>,
    pub data_units_written: Option<i64>,
    pub percentage_used: Option<i64>,
    pub available_spare: Option<i64>,
    pub unsafe_shutdowns: Option<i64>,
    pub integrity_errors: Option<i64>,
    pub ssd_temp_c: Option<i64>,
    pub cycle_count: Option<i64>,
    pub max_capacity_pct: Option<i64>,
    pub design_capacity: Option<i64>,
    pub condition: Option<String>,
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<Row>> {
    let mut stmt = conn.prepare(
        "SELECT ts, power_on_hours, power_cycles, data_units_read, data_units_written,
                percentage_used, available_spare, unsafe_shutdowns, integrity_errors,
                ssd_temp_c, cycle_count, max_capacity_pct, design_capacity, condition
         FROM snapshots ORDER BY ts DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(Row {
                ts: row.get(0)?,
                power_on_hours: row.get(1)?,
                power_cycles: row.get(2)?,
                data_units_read: row.get(3)?,
                data_units_written: row.get(4)?,
                percentage_used: row.get(5)?,
                available_spare: row.get(6)?,
                unsafe_shutdowns: row.get(7)?,
                integrity_errors: row.get(8)?,
                ssd_temp_c: row.get(9)?,
                cycle_count: row.get(10)?,
                max_capacity_pct: row.get(11)?,
                design_capacity: row.get(12)?,
                condition: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn all(conn: &Connection) -> Result<Vec<Row>> {
    recent(conn, i32::MAX as usize)
}
