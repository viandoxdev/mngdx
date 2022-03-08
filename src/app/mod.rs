use crossterm::event;
use render::render;
use std::sync::{
    atomic::AtomicBool,
    mpsc::{self, Sender},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tui::{backend::Backend, Terminal};

use self::{events::AppEvent, render::NS_PER_FRAME};

mod events;
mod render;

pub struct AppData {
    block_name: String,
}

pub struct App<B: Backend> {
    data: Arc<Mutex<AppData>>,
    terminal: Arc<Mutex<Terminal<B>>>,
    event_queue: Sender<AppEvent>,
}

impl<B: 'static> App<B>
where
    B: Backend + Send,
{
    pub fn new(terminal: Terminal<B>) -> Self {
        App {
            data: Arc::new(Mutex::new(AppData {
                block_name: "?".to_string(),
            })),
            terminal: Arc::new(Mutex::new(terminal)),
            // dummy channel
            event_queue: mpsc::channel().0,
        }
    }
    pub fn run(&mut self) {
        // boolean shared across all 3 threads to know when to stop.
        let stop = Arc::new(AtomicBool::new(false));

        // those will be moved

        let data_mutex = self.data.clone();
        let terminal_mutex = self.terminal.clone();
        // weird name but needs to be different to not move stop.
        let should_stop = stop.clone();

        // rendering thread
        thread::spawn(move || {
            loop {
                // Idk wtf this all means but atomic sounds cool
                if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                {
                    // lock access to data and terminal
                    let data_guard = data_mutex.lock();
                    let terminal_guard = terminal_mutex.lock();

                    // try to render
                    if let (Ok(data), Ok(mut terminal)) = (data_guard, terminal_guard) {
                        let _ = terminal.draw(|f| render(f, &data));
                    }
                }

                // wait until next frame
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let since_last_frame = now % NS_PER_FRAME;
                let sleep = NS_PER_FRAME - since_last_frame;

                thread::sleep(Duration::from_nanos(sleep as u64));
            }
        });

        // these will be moved
        let data_mutex = self.data.clone();
        let should_stop = stop.clone();

        // create event channel / queue
        let (sender, receiver) = mpsc::channel();

        self.event_queue = sender;

        // main thread
        thread::spawn(move || {
            loop {
                if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Ok(event) = receiver.recv() {
                    // unwrap because PoisonErrors only happen after a thread crashed, and by that
                    // point, something much bigger has fucked up.
                    let mut data = data_mutex.lock().unwrap();

                    events::process_event(event, &mut data, &should_stop);
                }
            }
        });

        // input "thread", has to be on main because ¯\_(ツ)_/¯
        loop {
            if stop.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            if let Some(event) = Self::event() {
                let _ = self.event_queue.send(event);
            }
        }
    }

    /// get a mutex guard to the terminal
    pub fn get_terminal(&self) -> Option<MutexGuard<Terminal<B>>> {
        self.terminal.lock().ok()
    }

    /// tries to read and convert an event into an AppEvent.
    pub fn event() -> Option<AppEvent> {
        // if no event in 50ms
        if !event::poll(Duration::from_millis(50)).ok()? {
            return None;
        }

        let event = event::read().ok()?;
        event.try_into().ok()
    }
}
