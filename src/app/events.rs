use crate::api::Api;

use super::AppComponents;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rand::prelude::SliceRandom;
use std::{
    io::Write,
    sync::{atomic::AtomicBool, Arc},
};
use tui::backend::Backend;
use uuid::Uuid;

pub enum AppEvent {
    Dummy(String),
    Start,
    Quit,
    ReloadImages,
}

pub fn process_event<B: Backend + Write + Send + 'static>(
    event: AppEvent,
    mut comps: AppComponents<B>,
    should_stop: &Arc<AtomicBool>,
) {
    match event {
        AppEvent::Start => {
            let components = comps.clone();
            comps.task_producer.schedule(async move {
                let mut api = Api::new();
                let chapters = api
                    .manga_chapters(
                        Uuid::parse_str("e78a489b-6632-4d61-b00b-5206f5b8b22b").unwrap(),
                    )
                    .await
                    .unwrap();
                let chapter = chapters.choose(&mut rand::thread_rng()).unwrap();
                let pages = api.chapter_pages(*chapter).await.unwrap();

                comps.reader.lock().unwrap().read(pages, components);
            });
        }
        AppEvent::Dummy(s) => {
            comps.state.lock().unwrap().block_name = s;
        }
        AppEvent::Quit => {
            should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        AppEvent::ReloadImages => {
            let _ = comps.image_manager.lock().unwrap().force_reload_images();
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
