use crate::app::render::FRAME;
use crate::consts::EXECUTOR_THREAD_COUNT;
use crate::images;
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

use self::state::AppState;
use self::{
    events::AppEvent,
    render::{render_images, render_widgets},
};

mod events;
mod render;
mod state;
pub mod time;

pub struct App<B: Backend> {
    state: Arc<Mutex<AppState>>,
    terminal: Arc<Mutex<Terminal<B>>>,
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

// schedules a task to be run by the executor thread
pub fn schedule_task<F: Future<Output = ()> + Send + 'static>(
    task_producer: &mut Sender<Pin<Box<dyn Future<Output = ()> + Send>>>,
    f: F,
) {
    task_producer.send(Box::pin(f)).unwrap();
}

impl<B: 'static> App<B>
where
    B: Backend + Send + Write,
{
    pub fn new(terminal: Terminal<B>, fd: RawFd) -> Self {
        App {
            state: Arc::new(Mutex::new(AppState::new())),
            terminal: Arc::new(Mutex::new(terminal)),
            fd,
        }
    }

    pub fn run(&mut self) {
        let (event_producer, event_receciver) = mpsc::channel();
        let (mut task_producer, task_receiver) =
            mpsc::channel::<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>();
        // boolean shared across all threads to know when to stop.
        let stop = Arc::new(AtomicBool::new(false));

        event_producer.send(AppEvent::Start).unwrap();

        // Rendering thread
        {
            let state_mutex = self.state.clone();
            let terminal_mutex = self.terminal.clone();
            let fd = self.fd;
            let should_stop = stop.clone();

            spawn_named("Render", move || {
                let mut time = Vec::with_capacity(600);
                let mut last_frame;
                loop {
                    if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    {
                        last_frame = Instant::now();

                        let ws = images::get_terminal_winsize(fd).unwrap();
                        // lock access to state and terminal
                        let state_guard = state_mutex.lock();
                        let terminal_guard = terminal_mutex.lock();

                        // try to render
                        if let (Ok(mut state), Ok(mut terminal)) = (state_guard, terminal_guard) {
                            let _ = terminal.draw(|f| render_widgets(f, &state));
                            let _ = render_images(&mut terminal, &mut state, &ws);
                        }
                    }

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
            let state_mutex = self.state.clone();
            let terminal_mutex = self.terminal.clone();
            let should_stop = stop.clone();
            spawn_named("Event Loop", move || loop {
                if should_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                if let Ok(event) = event_receciver.recv() {
                    events::process_event(
                        event,
                        state_mutex.clone(),
                        terminal_mutex.clone(),
                        &should_stop,
                        &mut task_producer,
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
