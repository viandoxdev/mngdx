use std::{
    io::Write,
    lazy::SyncLazy,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    consts::FRAME_RATE,
    images::{ImageManagerTerminalExt, TermWinSize},
};

use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

use super::state::AppState;

pub static FRAME: SyncLazy<Duration> = SyncLazy::new(|| Duration::from_secs(1) / FRAME_RATE);

pub fn render_widgets<B: Backend>(f: &mut Frame<B>, state: &AppState) {
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
        .title(state.block_name.clone())
        .borders(Borders::ALL);
    let d = Block::default()
        .title(format!(
            "{} {size:?}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
        .borders(Borders::ALL);
    f.render_widget(title, layout[0]);
    f.render_widget(t, layout[1]);
    f.render_widget(d, layout[2]);
}

pub fn render_images<B: Backend + Write>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
    ws: &TermWinSize,
) -> Result<()> {
    let _ = state.image_manager.display_image_best_fit(
        terminal,
        1,
        Rect {
            x: 0,
            y: 0,
            width: ws.cols,
            height: ws.rows,
        },
        ws,
    );
    Ok(())
}
