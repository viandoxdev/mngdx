use std::{
    borrow::BorrowMut,
    io::Write,
    lazy::SyncLazy,
    time::{Duration, SystemTime, UNIX_EPOCH}, sync::Arc,
};

use crate::{consts::FRAME_RATE, images::TermWinSize};

use anyhow::Result;
use parking_lot::Mutex;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::{state::AppState, AppComponents, reader::Reader};

pub static FRAME: SyncLazy<Duration> = SyncLazy::new(|| Duration::from_secs(1) / FRAME_RATE);

pub fn render_widgets<B: Backend + Write + Send>(f: &mut Frame<B>, state: &AppState, reader: Arc<Mutex<dyn Reader<B>>>) -> Rect {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Percentage(80),
            Constraint::Min(10),
        ])
        .split(size);
    let title = Paragraph::new("mngdx")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    let t = Paragraph::new(format!("loading...\n{}", reader.lock().current()))
        .alignment(Alignment::Center)
        .block(Block::default());
    let d = Block::default()
        .title(format!(
            "{} {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            state.block_name
        ))
        .borders(Borders::LEFT);
    f.render_widget(title, layout[0]);
    f.render_widget(t, layout[1]);
    f.render_widget(d, layout[2]);

    layout[1]
}

pub fn render<B: Backend + Write + Send + 'static>(
    comps: AppComponents<B>,
    ws: &TermWinSize,
) -> Result<()> {
    let mut reader_area = Rect {
        x: 0,
        y: 0,
        width: ws.cols,
        height: ws.rows,
    };

    comps
        .terminal
        .lock()
        .draw(|f| reader_area = render_widgets(f, comps.state.lock().borrow_mut(), comps.reader.clone()))?;

    comps.reader.lock().draw(
        reader_area,
        ws,
        comps.terminal.lock().borrow_mut(),
        comps.image_manager.lock().borrow_mut(),
    )?;
    comps
        .image_manager
        .lock()
        .draw(comps.terminal.lock().backend_mut())?;
    Ok(())
}
