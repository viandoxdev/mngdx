#![allow(dead_code)]

mod api;
mod app;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

// TODO: remove viuer and replace with custom terminal image display tool taking adventage of
// kitty's protocol. For example: loading an image with an id and then changing its position
// without reloading it (probably necessary to achieve 60 fps rendering, and also cool to cut down
// on dependencies).

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();

    // build app and pass terminal to it
    let mut app = App::new(terminal);

    app.run();

    // get terminal from app to reset back to normal
    let terminal = &mut app.get_terminal().unwrap();

    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}
