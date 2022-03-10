use std::{time::{SystemTime, UNIX_EPOCH}, io::Write};

use crate::images;

use super::AppData;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

const FRAME_RATE: f64 = 60.0;
pub const NS_PER_FRAME: u128 = (1.0 / FRAME_RATE * 1_000_000_000.0) as u128;

pub fn render_widgets<B: Backend>(f: &mut Frame<B>, data: &AppData) {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Min(10),
        ])
        .split(size);
    let title = Paragraph::new("mngdx")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    let t = Block::default()
        .title(data.block_name.clone())
        .borders(Borders::ALL);
    let d = Block::default()
        .title(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string(),
        )
        .borders(Borders::ALL);
    f.render_widget(title, layout[0]);
    f.render_widget(t, layout[1]);
    f.render_widget(d, layout[2]);
}

pub fn render_images<B: Backend + Write>(terminal: &mut Terminal<B>, data: &AppData) -> Result<()> {
    terminal.set_cursor(0, 0)?;
    let stdout = terminal.backend_mut();
    images::display_image(stdout, 1, 1, 30, 10)?;
    Ok(())
}
