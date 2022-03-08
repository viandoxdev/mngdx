use std::sync::{Arc, atomic::AtomicBool};
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};
use super::AppData;

pub enum AppEvent {
    Dummy(String),
    Quit,
}

pub fn process_event(event: AppEvent, data: &mut AppData, should_stop: &Arc<AtomicBool>) {
    match event {
        AppEvent::Dummy(s) => {
            data.block_name = s;
        }
        AppEvent::Quit => {
            should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
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

            Event::Mouse(me) => Ok(AppEvent::Dummy(format!("{me:?}"))),

            _ => Err(anyhow::Error::msg("No App event for this event"))
        }
    }
}
