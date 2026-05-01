# decay

**Are you riding your machine until the wheels fall off?**

`decay` is a cross-platform CLI tool that tracks SSD wear, battery health, CPU temperature, and disk I/O over time. Like a car odometer — but for your laptop's components.

Take daily snapshots. Watch the trend lines. Know when the wheels are about to come off.

Works on **macOS**, **Linux**, and **Windows**.

![decay demo](promo/demo.gif)

## Install

```bash
# macOS
brew install smartmontools
cargo install decay

# Linux (Debian/Ubuntu)
sudo apt install smartmontools
cargo install decay

# Linux (Fedora/RHEL)
sudo dnf install smartmontools
cargo install decay

# Windows (with Chocolatey)
choco install smartmontools
cargo install decay
```

## Quick start

```bash
decay snapshot   # take your first reading
decay            # see current health + sparklines
decay install    # set up daily automatic snapshots
```

## Commands

```bash
# See current health + sparklines + mileage predictions
decay

# Take a snapshot (or let `decay install` do it daily)
decay snapshot

# Interactive TUI chart — arrow keys to switch tabs, q to quit
decay chart

# Predict when components hit critical thresholds
decay predict

# View snapshot history
decay history

# Set up / remove daily automatic snapshots (macOS launchd)
decay install
decay uninstall

# Export all data
decay export --format json
decay export --format csv
```

## What it tracks

All numbers come from firmware or OS sensors — they persist across reboots and can't be faked.

**SSD** (via `smartctl` — NVMe + SATA):
- Percentage used / available spare
- Data written & read (lifetime TB)
- Power cycles, unsafe shutdowns
- Integrity errors, temperature

**Battery**:
- Cycle count, max capacity %, condition
- macOS: `ioreg` + `system_profiler`
- Linux: `/sys/class/power_supply/BAT*/`
- Windows: `wmic Win32_Battery`

**System health**:
- CPU temperature (per-snapshot trend)
- Disk I/O benchmark (64 MB sequential read/write, MB/s)

## Example output

```
  🚗 decay — how many miles left?

  SSD  APPLE SSD AP0512Z
    Wear: 0%  ▁▁▁▁▁▁▁▁  Spare: 100%  Temp: 28°C
    Written: 4.08 TB  Read: 3.06 TB  Cycles: 131
    Unsafe shutdowns: 5  Integrity errors: 0

  Battery
    Health: 100%  ▁▁▁▁  Cycles: 42 / 1000  Condition: Normal
    Design capacity: 8,579 mAh

  System
    CPU temp: 28°C  ▁
    Disk I/O: 14598 MB/s read  3308 MB/s write  ▁

  🛞 SSD: SSD wear is flat — cruising with no visible degradation
  🛞 Battery: Battery health is steady — no degradation trend yet
```

## How it works

1. `decay snapshot` reads firmware counters (SSD via `smartctl`, battery via OS APIs), measures CPU temp, and runs a disk I/O benchmark
2. Stores each reading in a local SQLite database (`~/.local/share/decay/decay.db`)
3. `decay` renders the latest snapshot with sparklines and mileage predictions
4. `decay chart` opens an interactive TUI with 6 time-series tabs
5. `decay install` creates a macOS LaunchAgent for daily automatic snapshots

No network calls. No telemetry. Everything stays on your machine.

## Requirements

- [smartmontools](https://www.smartmontools.org/) (for SSD data)
- Rust toolchain (to build from source)

| Platform | SSD | Battery | CPU Temp | Disk I/O |
|----------|-----|---------|----------|----------|
| macOS | `smartctl` | `ioreg` + `system_profiler` | `ioreg` | sequential bench |
| Linux | `smartctl` | `/sys/class/power_supply/` | `/sys/class/thermal/` | sequential bench |
| Windows | `smartctl` | `wmic` | `wmic` | sequential bench |

## Roadmap

- [x] `decay chart` — interactive TUI with 6 time-series tabs
- [x] `decay predict` — project when SSD/battery hit critical thresholds
- [x] `decay install` — daily automatic snapshots via launchd
- [x] CSV export
- [x] Cross-platform support (macOS, Linux, Windows)
- [x] CPU temperature tracking
- [x] Disk I/O benchmark per snapshot
- [ ] Homebrew formula
- [ ] crates.io publish
- [ ] GitHub Actions CI
- [ ] SMART change alerts

## License

MIT
