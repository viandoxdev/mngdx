use crate::api::Api;

use super::{schedule_task, state::AppState};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rand::prelude::SliceRandom;
use std::{
    future::Future,
    io::Write,
    pin::Pin,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
};
use tui::{backend::Backend, Terminal};
use uuid::Uuid;

pub enum AppEvent {
    Dummy(String),
    Start,
    Quit,
    ReloadImages,
}

pub fn process_event<B: Backend + Write + Send + 'static>(
    event: AppEvent,
    state: Arc<Mutex<AppState>>,
    terminal: Arc<Mutex<Terminal<B>>>,
    should_stop: &Arc<AtomicBool>,
    task_producer: &mut Sender<Pin<Box<dyn Future<Output = ()> + Send>>>,
) {
    match event {
        AppEvent::Start => {
            schedule_task(task_producer, async move {
                let mut api = Api::new();
                let chapters = api
                    .manga_chapters(
                        Uuid::parse_str("e78a489b-6632-4d61-b00b-5206f5b8b22b").unwrap(),
                    )
                    .await
                    .unwrap();
                let chapter = chapters.choose(&mut rand::thread_rng()).unwrap();
                let pages = api.chapter_pages(*chapter).await.unwrap();
                let page = pages.choose(&mut rand::thread_rng()).unwrap();
                let bytes = reqwest::get(page).await.unwrap().bytes().await.unwrap();

                let image = image::load_from_memory(&bytes).unwrap();
                state.lock().unwrap().image_manager.add_image(1, image);
                state.lock().unwrap().block_name = "LOADED".to_string();
            });
        }
        AppEvent::Dummy(s) => {
            state.lock().unwrap().block_name = s;
        }
        AppEvent::Quit => {
            should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        AppEvent::ReloadImages => {
            let _ = state.lock().unwrap().image_manager.force_reload_images();
        }
    }
}

impl TryFrom<Event> for AppEvent {
    type Error = anyhow::Error;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => Ok(AppEvent::Quit),

            Event::Resize(_, _)
            | Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
            }) => Ok(AppEvent::ReloadImages),

            Event::Mouse(me) => Ok(AppEvent::Dummy(format!("{me:?}"))),

            _ => Err(anyhow::Error::msg("No App event for this event")),
        }
    }
}
