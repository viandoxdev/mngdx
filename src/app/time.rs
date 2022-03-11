#[cfg(feature = "timers")]
use std::{
    cell::RefCell,
    collections::HashMap,
    time::{Duration, Instant},
};

pub trait IntoString {
    fn into_string(self) -> String;
}

impl<T: ToString> IntoString for T {
    default fn into_string(self) -> String {
        self.to_string()
    }
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}

#[cfg(feature = "timers")]
struct Timer {
    timers: HashMap<String, Instant>,
}

#[cfg(feature = "timers")]
impl Timer {
    fn new() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }

    fn start(&mut self, name: impl IntoString) {
        self.timers.insert(name.into_string(), Instant::now());
    }

    fn stop(&mut self, name: impl IntoString) -> Option<Duration> {
        let name = name.into_string();
        if let Some(i) = self.timers.remove(&name) {
            let d = i.elapsed();
            log::debug!("{name} took {:.8}s", d.as_secs_f64());
            Some(d)
        } else {
            log::warn!("Stopping non-existant timer {name}");
            None
        }
    }
}

#[cfg(feature = "timers")]
thread_local! {
    static TIMER: RefCell<Timer> = RefCell::new(Timer::new());
}

#[cfg(feature = "timers")]
#[inline]
pub fn timer_start(name: impl IntoString) {
    TIMER.with(|t| {
        t.borrow_mut().start(name);
    });
}

#[cfg(feature = "timers")]
#[inline]
pub fn timer_stop(name: impl IntoString) -> Option<Duration> {
    TIMER.with(|t| t.borrow_mut().stop(name))
}

#[cfg(not(feature = "timers"))]
#[inline]
pub fn timer_start(_: impl IntoString) {}

#[cfg(not(feature = "timers"))]
#[inline]
pub fn timer_stop(_: impl IntoString) {}
