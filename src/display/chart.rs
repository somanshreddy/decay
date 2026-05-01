use crate::db::Row;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};
use std::io;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    SsdWear,
    SsdWritten,
    BatteryHealth,
    BatteryCycles,
    CpuTemp,
    DiskIO,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::SsdWear, Tab::SsdWritten, Tab::BatteryHealth, Tab::BatteryCycles, Tab::CpuTemp, Tab::DiskIO]
    }

    fn label(&self) -> &'static str {
        match self {
            Tab::SsdWear => "SSD Wear %",
            Tab::SsdWritten => "SSD Written",
            Tab::BatteryHealth => "Battery Health %",
            Tab::BatteryCycles => "Battery Cycles",
            Tab::CpuTemp => "CPU Temp °C",
            Tab::DiskIO => "Disk I/O MB/s",
        }
    }

    fn next(&self) -> Tab {
        let tabs = Tab::all();
        let idx = tabs.iter().position(|t| t == self).unwrap_or(0);
        tabs[(idx + 1) % tabs.len()]
    }

    fn prev(&self) -> Tab {
        let tabs = Tab::all();
        let idx = tabs.iter().position(|t| t == self).unwrap_or(0);
        tabs[(idx + tabs.len() - 1) % tabs.len()]
    }
}

struct ChartData {
    points: Vec<(f64, f64)>,
    y_min: f64,
    y_max: f64,
    x_min: f64,
    x_max: f64,
    unit: &'static str,
}

fn extract_data(rows: &[Row], tab: Tab) -> ChartData {
    let points: Vec<(f64, f64)> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            let y = match tab {
                Tab::SsdWear => r.percentage_used? as f64,
                Tab::SsdWritten => {
                    r.data_units_written? as f64 * 512.0 * 1000.0 / 1e12
                }
                Tab::BatteryHealth => r.max_capacity_pct? as f64,
                Tab::BatteryCycles => r.cycle_count? as f64,
                Tab::CpuTemp => r.cpu_temp_c? as f64,
                Tab::DiskIO => r.disk_read_mbs? as f64,
            };
            Some((i as f64, y))
        })
        .collect();

    if points.is_empty() {
        return ChartData {
            points: vec![(0.0, 0.0)],
            y_min: 0.0,
            y_max: 100.0,
            x_min: 0.0,
            x_max: 1.0,
            unit: "",
        };
    }

    let y_vals: Vec<f64> = points.iter().map(|p| p.1).collect();
    let y_min = y_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let padding = ((y_max - y_min) * 0.1).max(1.0);

    let unit = match tab {
        Tab::SsdWear | Tab::BatteryHealth => "%",
        Tab::SsdWritten => "TB",
        Tab::BatteryCycles => "",
        Tab::CpuTemp => "°C",
        Tab::DiskIO => "MB/s",
    };

    ChartData {
        x_min: 0.0,
        x_max: (points.len() as f64 - 1.0).max(1.0),
        y_min: (y_min - padding).max(0.0),
        y_max: y_max + padding,
        points,
        unit,
    }
}

pub fn run(rows: &[Row]) -> Result<()> {
    if rows.is_empty() {
        println!("  No snapshots yet. Run `decay snapshot` first.");
        return Ok(());
    }

    // Reverse so oldest is first (rows come in DESC order)
    let rows: Vec<Row> = rows.iter().rev().cloned().collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tab = Tab::SsdWear;

    loop {
        terminal.draw(|f| draw(f, &rows, tab))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Right | KeyCode::Tab | KeyCode::Char('l') => tab = tab.next(),
                KeyCode::Left | KeyCode::Char('h') => tab = tab.prev(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw(f: &mut Frame, rows: &[Row], tab: Tab) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(1)])
        .split(f.area());

    // Tab bar
    let tab_titles: String = Tab::all()
        .iter()
        .map(|t| {
            if *t == tab {
                format!(" [{}] ", t.label())
            } else {
                format!("  {}  ", t.label())
            }
        })
        .collect();

    let tabs = Paragraph::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).title("🚗 decay chart"))
        .style(Style::default().fg(Color::White));
    f.render_widget(tabs, chunks[0]);

    // Chart
    let data = extract_data(rows, tab);
    let color = match tab {
        Tab::SsdWear => Color::Yellow,
        Tab::SsdWritten => Color::Cyan,
        Tab::BatteryHealth => Color::Green,
        Tab::BatteryCycles => Color::Magenta,
        Tab::CpuTemp => Color::Red,
        Tab::DiskIO => Color::LightBlue,
    };

    let dataset = Dataset::default()
        .name(tab.label())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(&data.points);

    let y_label_lo = format!("{:.1}{}", data.y_min, data.unit);
    let y_label_hi = format!("{:.1}{}", data.y_max, data.unit);

    let chart = Chart::new(vec![dataset])
        .block(Block::default().borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title(Span::styled("snapshots →", Style::default().dim()))
                .bounds([data.x_min, data.x_max]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(tab.label(), Style::default().fg(color)))
                .bounds([data.y_min, data.y_max])
                .labels(vec![ratatui::text::Line::from(y_label_lo), ratatui::text::Line::from(y_label_hi)]),
        );
    f.render_widget(chart, chunks[1]);

    // Help bar
    let help = Paragraph::new(" ←/→ switch tab  q quit")
        .style(Style::default().dim());
    f.render_widget(help, chunks[2]);
}

