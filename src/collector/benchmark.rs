use super::Snapshot;
use anyhow::Result;
use std::io::{Read, Write};
use std::time::Instant;

const BENCH_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub fn collect_benchmark() -> Result<Snapshot> {
    let dir = bench_dir()?;
    let path = dir.join("decay_bench.tmp");

    let write_mbs = bench_write(&path)?;
    let read_mbs = bench_read(&path)?;

    let _ = std::fs::remove_file(&path);

    Ok(Snapshot {
        disk_write_mbs: Some(write_mbs),
        disk_read_mbs: Some(read_mbs),
        ..Default::default()
    })
}

fn bench_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let dir = std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("decay");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn bench_write(path: &std::path::Path) -> Result<i64> {
    let data = vec![0xABu8; BENCH_SIZE];
    let start = Instant::now();
    let mut f = std::fs::File::create(path)?;
    f.write_all(&data)?;
    f.sync_all()?;
    let elapsed = start.elapsed().as_secs_f64();
    let mbs = (BENCH_SIZE as f64 / 1024.0 / 1024.0) / elapsed;
    Ok(mbs as i64)
}

fn bench_read(path: &std::path::Path) -> Result<i64> {
    // Flush OS page cache as much as possible by reopening
    let start = Instant::now();
    let mut f = std::fs::File::open(path)?;
    let mut buf = vec![0u8; BENCH_SIZE];
    f.read_exact(&mut buf)?;
    let elapsed = start.elapsed().as_secs_f64();
    let mbs = (BENCH_SIZE as f64 / 1024.0 / 1024.0) / elapsed;
    Ok(mbs as i64)
}
