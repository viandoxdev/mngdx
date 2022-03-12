// There has to be a cleaner way, but whatever

// The padding to use when supported and enabled.
#[cfg(feature = "set_padding")]
pub const KITTY_PADDING: i32 = 5;
// How many images can we have loaded at a time
// see https://sw.kovidgoyal.net/kitty/graphics-protocol/#image-persistence-and-storage-quotas
pub const IMAGE_SLOTS: u32 = 20;
// How many tasks can be run in parallel on the executor thread, setting this number too high will get you rate limited.
pub const EXECUTOR_THREAD_COUNT: u32 = 2;
// The framerate to aim for
pub const FRAME_RATE: u32 = 60;
