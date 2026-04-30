# decay

**Are you riding your Mac until the wheels fall off?**

`decay` is a CLI tool that tracks your Mac's SSD wear and battery health over time. Like a car odometer — but for your laptop's components.

Take daily snapshots. Watch the trend lines. Know when the wheels are about to come off.

## Install

```bash
brew install smartmontools  # required for SSD data
cargo install decay
```

## Usage

```bash
# See current health + sparklines
decay

# Take a snapshot (run daily)
decay snapshot

# View history
decay history

# Export all data as JSON
decay export --format json
```

## What it tracks

All numbers come from firmware — they persist across reboots and can't be faked.

**SSD** (via `smartctl`):
- Percentage used / available spare
- Data written & read (lifetime TB)
- Power cycles, unsafe shutdowns
- Integrity errors, temperature

**Battery** (via `ioreg` + `system_profiler`):
- Cycle count
- Max capacity %
- Condition

## Example output

```
  🚗 decay — how many miles left?

  SSD  APPLE SSD AP0512Z
    Wear: 0%  ▁▁▁▁▁▁▁▁  Spare: 100%  Temp: 29°C
    Written: 4.07 TB  Read: 3.05 TB  Cycles: 131
    Unsafe shutdowns: 5  Integrity errors: 0

  Battery
    Health: 100%  ▇▇▇▇▇▇▇▇  Cycles: 42 / 1000  Condition: Normal
    Design capacity: 8,579 mAh

  🛞 Run `decay snapshot` daily — need more data for mileage estimates
```

## How it works

1. `decay snapshot` shells out to `smartctl` and `ioreg` to read firmware counters
2. Stores each reading in a local SQLite database (`~/.local/share/decay/decay.db`)
3. `decay` renders the latest snapshot with sparklines from your history

No network calls. No telemetry. Everything stays on your machine.

## Requirements

- macOS (Apple Silicon or Intel)
- [smartmontools](https://www.smartmontools.org/) (`brew install smartmontools`)
- Rust toolchain (to build from source)

## Roadmap

- [ ] `decay chart` — interactive TUI with time-series charts
- [ ] `decay predict` — project when SSD/battery hit critical thresholds
- [ ] `decay install` — set up daily automatic snapshots via launchd
- [ ] Linux support
- [ ] Homebrew formula

## License

MIT
