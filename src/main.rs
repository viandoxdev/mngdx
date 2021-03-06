#![warn(clippy::disallowed_types)]
#![allow(dead_code)]
#![feature(once_cell)]
#![feature(min_specialization)]

mod api;
mod app;
pub mod consts;
mod images;

use std::{io, os::unix::prelude::AsRawFd};
#[cfg(feature = "set_padding")]
use {consts::KITTY_PADDING, std::process::Command};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut stdout = io::stdout();

    #[cfg(feature = "set_padding")]
    let _ = Command::new("kitty")
        .arg("@set-spacing")
        .arg(format!("padding={KITTY_PADDING}"))
        .output()
        .map_err(|err| log::warn!("Error trying to set padding in kitty ({err:?})"));

    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let fd = stdout.as_raw_fd();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    // build app and pass terminal to it
    let mut app = App::new(terminal, fd);

    app.run();

    // get terminal from app to reset back to normal
    let terminal = &mut app.get_terminal();

    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    #[cfg(feature = "set_padding")]
    let _ = Command::new("kitty")
        .arg("@set-spacing")
        .arg("padding=default")
        .output()
        .map_err(|err| log::warn!("Error trying to reset padding in kitty ({err:?})"));

    Ok(())
}
