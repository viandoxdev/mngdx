#![allow(dead_code)]
#![feature(once_cell)]

mod api;
mod app;
mod images;

use std::io;

use anyhow::Result;
use api::Api;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};
use uuid::Uuid;

use crate::app::App;

// TODO: remove viuer and replace with custom terminal image display tool taking adventage of
// kitty's protocol. For example: loading an image with an id and then changing its position
// without reloading it (probably necessary to achieve 60 fps rendering, and also cool to cut down
// on dependencies).
//
// DONE
//
// TODO: add image data on app data (or somewhere else): A struct written to by the main thread,
// read by the render thread, (RwLock) which holds: the ten images slots + terminal size (see about
// querying it on render to update every frame, but probably impossible because needs raw handle of
// stdout. Or do it in render thread but not function might be possible if raw handle is accessible
// from backend, so probably not). Either both the term size and images slots on the same struct or
// in separate (depending on if the term size is updated on a per frame basis).
// Or just run only ioctls on main except on input ? (unsafe risk ?).

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut api = Api::new();
    let pages = api.chapter_pages(Uuid::parse_str("00ec20cc-9e5e-49ef-bc54-09ae980ecaa1")?).await?;
    let page = pages.get(pages.len() / 2).unwrap();
    let bytes = reqwest::get(page).await?.bytes().await?;
    let image = image::load_from_memory(&bytes)?;
    let mut stdout = io::stdout();

    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    images::load_image(terminal.backend_mut(), 1, &image)?;

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

    Ok(())
}
