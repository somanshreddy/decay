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
cargo install --git https://github.com/somanshreddy/decay

# Linux (Debian/Ubuntu)
sudo apt install smartmontools
cargo install --git https://github.com/somanshreddy/decay

# Linux (Fedora/RHEL)
sudo dnf install smartmontools
cargo install --git https://github.com/somanshreddy/decay

# Windows (with Chocolatey)
choco install smartmontools
cargo install --git https://github.com/somanshreddy/decay
```

Once installed, decay keeps itself up to date — see [Updating](#updating).

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

# Version and updates
decay --version
decay update           # update now
decay update --check   # just see if a new release exists
```

## Updating

decay updates itself the same way `claude` does: every run checks the repo for a
newer release tag in the background, and when it finds one it installs it
without blocking whatever you were doing. The next command you run is the new
version.

```
  ⬆️  Updating decay 0.1.0 → 0.2.0 in the background. The next run uses the new version.
```

Details:

- The check is a `git ls-remote` against this repo (~0.5s) on a background
  thread, capped at 5 seconds. If you're offline it stays silent.
- The install is `cargo install --git … --tag … --force`, detached, logged to
  `~/.local/share/decay/update.log`.
- A failed install won't retry more than once every 6 hours.
- Only the cargo-installed binary auto-updates. A local `cargo run` /
  `target/debug` build never overwrites itself.
- The daily launchd job updates too — `decay install` gives it a PATH that
  includes `~/.cargo/bin` (and `/opt/homebrew/bin`, so scheduled snapshots find
  `smartctl`). Re-run `decay install` to pick that up on an existing schedule.

To turn it off, set `DECAY_NO_UPDATE=1`:

```bash
export DECAY_NO_UPDATE=1   # in your shell profile
```

Releases are semver tags (`v0.2.0`) on this repo; `decay --version` tells you
what you're running.

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
  🚗 decay — how many miles left?  v0.1.0

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

No telemetry. Every reading stays on your machine — the only network call decay
ever makes is the update check against its own GitHub repo, which sends nothing
but the request itself and can be disabled with `DECAY_NO_UPDATE=1`.

## Requirements

- [smartmontools](https://www.smartmontools.org/) (for SSD data)
- Rust toolchain (to build from source)
- `git` (for install and for the update check)

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
- [x] Self-updating releases (`decay update`)
- [ ] Homebrew formula
- [ ] crates.io publish
- [ ] GitHub Actions CI
- [ ] SMART change alerts

## Releasing

`scripts/release.sh` bumps the version, runs the tests, and pushes the commit
and tag. Pushing the tag is what ships the release — installed copies find it on
their next run.

```bash
./scripts/release.sh 0.2.0            # bump, test, commit, tag v0.2.0, push
./scripts/release.sh 0.2.0 --dry-run  # everything except commit/tag/push
```

Passing the version already in `Cargo.toml` tags it as-is, which is how the
first release gets cut.

## License

MIT
