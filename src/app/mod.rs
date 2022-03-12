use crate::app::render::FRAME;
use crate::consts::EXECUTOR_THREAD_COUNT;
use crate::images::{self, ImageManager};
use anyhow::{Result, Error};
use crossterm::event;
use std::future::Future;
use std::pin::Pin;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::{
    io::Write,
    os::unix::prelude::RawFd,
    sync::{
        atomic::AtomicBool,
        mpsc::{self, Sender},
        Arc, Mutex, MutexGuard,
    },
};
use tui::{backend::Backend, Terminal};

use self::reader::{Reader, PageReader};
use self::render::render;
use self::state::AppState;
use self::{
    events::AppEvent,
    render::{render_images, render_widgets},
};

mod events;
mod render;
mod state;
pub mod reader;
pub mod time;

pub struct App<B: Backend + Write + Send> {
    components: AppComponents<B>,
    fd: RawFd,
}

fn spawn_named<F, T>(name: impl ToString, f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .unwrap()
}

#[derive(Clone)]
pub struct TaskProducer {
    inner: Option<Sender<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl TaskProducer {
    pub fn new() -> Self {
        Self {
            inner: None,
        }
    }
    pub fn init(&mut self, sender: Sender<Pin<Box<dyn Future<Output = ()> + Send>>>) {
        self.inner.replace(sender);
    }
    pub fn schedule<F: Future<Output = ()> + Send + 'static>(&mut self, task: F) -> Result<()> {
        self.inner.as_mut().ok_or_else(|| Error::msg("Task scheduled on uninitialized producer."))?
            .send(Box::pin(task)).map_err(|_| Error::msg("Error when scheduling task."))
    }
}

pub struct AppComponents<B: Backend + Send + Write> {
    pub terminal: Arc<Mutex<Terminal<B>>>,
    pub state: Arc<Mutex<AppState>>,
    pub reader: Arc<Mutex<dyn Reader<B> + Send>>,
    pub image_manager: Arc<Mutex<ImageManager>>,
    pub task_producer: TaskProducer,
}

// Can't derive clone because derive thinks T needs Clone to clone Arc<T>,
// and assumes that B must be Clone, when it clearly doesn't have to.
impl<B: Backend + Send + Write> Clone for AppComponents<B> {
    fn clone(&self) -> Self {
        Self {
            terminal: self.terminal.clone(),
            state: self.state.clone(),
            reader: self.reader.clone(),
            image_manager: self.image_manager.clone(),
            task_producer: self.task_producer.clone(),
        }
    }
}

impl<B: 'static> App<B>
where
    B: Backend + Send + Write,
{
    pub fn new(terminal: Terminal<B>, fd: RawFd) -> Self {
        App {
            components: AppComponents {
                state: Arc::new(Mutex::new(AppState::new())),
                terminal: Arc::new(Mutex::new(terminal)),
                reader: Arc::new(Mutex::new(PageReader::new())),
                image_manager: Arc::new(Mutex::new(ImageManager::new())),
                task_producer: TaskProducer::new(),
            },
            fd,
        }
    }

    pub fn run(&mut self) {
        let (event_producer, event_receciver) = mpsc::channel();
        let (task_sender, task_receiver) =
            mpsc::channel::<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>();

        self.components.task_producer.init(task_sender);

        // boolean shared across all threads to know when to stop.
        let stop = Arc::new(AtomicBool::new(false));

        event_producer.send(AppEvent::Start).unwrap();

        // Rendering thread
        {
            let comps = self.components.clone();
            let fd = self.fd;
            let should_stop = stop.clone();

            spawn_named("Render", move || {
                let mut time = Vec::with_capacity(600);
                let mut last_frame;
                loop {
                    if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    last_frame = Instant::now();
                    let ws = images::get_terminal_winsize(fd).unwrap();
                    render(comps.clone(), &ws);

                    // wait until next frame
                    let since_last_frame = last_frame.elapsed();
                    let sleep = FRAME.saturating_sub(since_last_frame);

                    time.push(since_last_frame);

                    if time.len() == time.capacity() {
                        let avg = time
                            .drain(0..time.len())
                            .map(|x| x.as_secs_f64())
                            .reduce(|a, x| a + x)
                            .unwrap()
                            / time.capacity() as f64;
                        log::debug!("Frame info: average render time: {:.9}s ({:.3}%) (for the last {} frames)", avg, avg / FRAME.as_secs_f64() * 100.0, time.capacity());
                    }

                    thread::sleep(sleep);
                }
            });
        }

        // Executor thread
        // This is essentially a thread pool to run futures.
        // This is used for example to load an image, because we just want to start it and do other
        // things while its running.
        {
            let should_stop = stop.clone();
            // Thread owning the tokio runtime
            spawn_named("Executor Owner", move || {
                // TODO change that probably to remove the two unsafe blocks. Its late rn im tired
                // I can't think of a better solution.

                // Raw pointers are used because I can't tell rust that this thread will outlive
                // the spawned threads, raw pointers are just here to tell rust: "trust me, those
                // don't point to null".

                let rt = Box::new(tokio::runtime::Runtime::new().unwrap());
                let rt_ptr = Box::into_raw(rt);

                let receiver = Arc::new(Mutex::new(task_receiver));
                let mut threads = Vec::with_capacity(EXECUTOR_THREAD_COUNT as usize);

                // spawn threads
                for i in 0..EXECUTOR_THREAD_COUNT {
                    let receiver = receiver.clone();
                    let should_stop = should_stop.clone();
                    let handle = unsafe { (*rt_ptr).handle() };

                    threads.push(spawn_named(format!("Executor {i}"), move || loop {
                        if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }

                        if let Ok(task) = receiver.lock().unwrap().recv() {
                            log::debug!("Executor {i} received task");
                            handle.block_on(async move {
                                task.await;
                            });
                        }
                        thread::sleep(Duration::from_millis(50));
                    }));
                }

                // join to make sure the runtime lives long enough
                for t in threads {
                    t.join().unwrap();
                }
                // drop runtime, AFTER all the other threads are done
                unsafe {
                    Box::from_raw(rt_ptr);
                }
            });
        }

        // Event loop thread
        {
            let comps = self.components.clone();
            let should_stop = stop.clone();
            spawn_named("Event Loop", move || loop {
                if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Ok(event) = event_receciver.recv() {
                    events::process_event(
                        event,
                        comps.clone(),
                        &should_stop,
                    );
                }
            });
        }

        // Input "thread", has to be on main because ¯\_(ツ)_/¯
        {
            loop {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Some(event) = Self::event() {
                    let _ = event_producer.send(event);
                }
            }
        }
    }

    /// get a mutex guard to the terminal
    pub fn get_terminal(&self) -> Option<MutexGuard<Terminal<B>>> {
        self.components.terminal.lock().ok()
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
