use anyhow::{Context, Result};
use std::path::PathBuf;

const LABEL: &str = "com.decay.snapshot";

fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", LABEL))
}

fn decay_bin() -> Result<String> {
    std::env::current_exe()
        .context("could not determine decay binary path")?
        .to_str()
        .map(String::from)
        .context("binary path is not valid UTF-8")
}

pub fn install() -> Result<()> {
    let bin = decay_bin()?;
    let path = plist_path();

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{bin}</string>
        <string>snapshot</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>12</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/decay.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/decay.log</string>
</dict>
</plist>"#,
        label = LABEL,
        bin = bin,
    );

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, plist).context("failed to write launchd plist")?;

    let path_str = path.to_str().context("plist path is not valid UTF-8")?;
    let status = std::process::Command::new("launchctl")
        .args(["load", path_str])
        .status()
        .context("failed to run launchctl load")?;

    if status.success() {
        println!("  ✅ Installed! decay will snapshot daily at noon.");
        println!("     Plist: {}", path.display());
        println!("     Log:   /tmp/decay.log");
    } else {
        println!("  ⚠️  Plist written but launchctl load failed (exit {}).", status);
        println!("     Try: launchctl load {}", path.display());
    }

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let path = plist_path();

    if !path.exists() {
        println!("  Nothing to uninstall — plist not found at {}", path.display());
        return Ok(());
    }

    if let Some(path_str) = path.to_str() {
        let _ = std::process::Command::new("launchctl")
            .args(["unload", path_str])
            .status();
    }

    std::fs::remove_file(&path).context("failed to remove plist")?;
    println!("  ✅ Uninstalled. Daily snapshots stopped.");
    Ok(())
}
