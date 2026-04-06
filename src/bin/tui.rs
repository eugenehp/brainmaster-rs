//! Real-time BrainMaster Discovery EEG terminal UI.

use std::io;
use std::time::{Duration, Instant};

use brainmaster::prelude::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

const DISPLAY_SAMPLES: usize = 512;
const DISPLAY_CHANNELS: usize = 8; // Show first 8 EEG channels

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let ports = DiscoveryDevice::find()?;
    if ports.is_empty() { eprintln!("No ports found."); return Ok(()); }

    println!("Connecting to {}...", ports[0]);
    let mut device = DiscoveryDevice::open(&ports[0])?;
    device.start()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut ring: Vec<[f32; NUM_CHANNELS]> = Vec::new();
    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    loop {
        // Read all available frames
        for _ in 0..10 {
            if let Ok(Some(_)) = device.read_frame() {
                if ring.len() >= DISPLAY_SAMPLES { ring.remove(0); }
                ring.push(device.channels().data);
            }
        }

        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(std::iter::once(Constraint::Length(3))
                    .chain((0..DISPLAY_CHANNELS).map(|_| Constraint::Min(3)))
                    .collect::<Vec<_>>())
                .split(area);

            let title = format!(" BrainMaster Discovery | {} Hz | {} ch | {} samples | q to quit ",
                SAMPLING_RATE, NUM_CHANNELS, ring.len());
            f.render_widget(Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Cyan)), chunks[0]);

            let colors = [Color::Green, Color::Yellow, Color::Blue, Color::Magenta, Color::Red, Color::Cyan, Color::White, Color::LightGreen];
            for ch in 0..DISPLAY_CHANNELS {
                let data: Vec<(f64, f64)> = ring.iter().enumerate().map(|(i, s)| (i as f64, s[ch] as f64)).collect();
                let y_min = data.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
                let y_max = data.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max);
                let margin = (y_max - y_min).max(1.0) * 0.1;

                let dataset = Dataset::default()
                    .name(CHANNEL_NAMES[ch])
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(colors[ch]))
                    .data(&data);

                let chart = Chart::new(vec![dataset])
                    .block(Block::default().borders(Borders::ALL).title(format!(" {} (µV) ", CHANNEL_NAMES[ch])))
                    .x_axis(Axis::default().bounds([0.0, DISPLAY_SAMPLES as f64]))
                    .y_axis(Axis::default().bounds([y_min - margin, y_max + margin])
                        .labels::<Vec<Line>>(vec![
                            format!("{:.0}", y_min - margin).into(),
                            format!("{:.0}", y_max + margin).into(),
                        ]));
                f.render_widget(chart, chunks[1 + ch]);
            }
        })?;

        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') { break; }
            }
        }
        if last_tick.elapsed() >= tick_rate { last_tick = Instant::now(); }
    }

    device.stop()?;
    device.close();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("Bye!");
    Ok(())
}
