use std::time::{SystemTime, UNIX_EPOCH};

use tui::{Frame, layout::{Direction, Constraint, Layout, Alignment}, backend::Backend, widgets::{Paragraph, Block, Borders, BorderType}, style::{Color, Style}};
use super::AppData;

const FRAME_RATE: f64 = 30.0;
pub const NS_PER_FRAME: u128 = (1.0 / FRAME_RATE * 1_000_000_000.0) as u128;

pub fn render<B: Backend>(f: &mut Frame<B>, data: &AppData) {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10), Constraint::Min(10)])
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
        .title(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string())
        .borders(Borders::ALL);
    f.render_widget(title, layout[0]);
    f.render_widget(t, layout[1]);
    f.render_widget(d, layout[2]);
}