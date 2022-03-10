use std::{io::Write, os::unix::prelude::AsRawFd};

use anyhow::{Result, Error};
use image::DynamicImage;

// this is the same as libc::winsize, but I didn't want libc to be exposed (and this implements
// debug)
#[derive(Debug)]
pub struct TermWinSize {
    width: u16,
    height: u16,
    cols: u16,
    rows: u16,
}

impl From<libc::winsize> for TermWinSize {
    fn from(v: libc::winsize) -> Self {
        Self {
            width: v.ws_xpixel, height: v.ws_ypixel,
            cols: v.ws_col, rows: v.ws_row,
        }
    }
}

/// get the terminal's size (pixels and cells). Uses ioctl (unsafe).
pub fn get_terminal_winsize(fd: impl AsRawFd) -> Result<TermWinSize> {
    let ws = libc::winsize {
        ws_xpixel: 0,
        ws_ypixel: 0,
        ws_col: 0,
        ws_row: 0,
    };
    if unsafe {
        libc::ioctl(fd.as_raw_fd(), libc::TIOCGWINSZ, &ws)
    } != 0 {
        Err(Error::msg("Non zero result for ioctl (TIOCGWINSZ)."))
    } else {
        Ok(ws.into())
    }
}

pub fn display_image(stdout: &mut impl Write, id: u32, pid: u32, w: u32, h: u32) -> Result<()> {
    write!(stdout, "\x1b_Ga=p,C=1,z=1073741824,i={id},p={pid},c={w},r={h};AAAA\x1b\\")?;
    stdout.flush()?;
    Ok(())
}

#[cfg(feature = "use_tempfiles")]
pub fn load_image(stdout: &mut impl Write, id: u32, image: &DynamicImage) -> Result<()> {
    let rgba8 = image.to_rgba8();
    let raw = rgba8.as_raw();
    let path = store_in_tmp_file(raw)?;

    write!(stdout, "\x1b_Gf=32,i={id},s={},v={},t=t;{}\x1b\\",
           image.width(), image.height(),
           base64::encode(path.to_str().ok_or(Error::msg("Could not convert path to tempfile to str."))?
    ))?;
    writeln!(stdout)?;
    stdout.flush()?;

    Ok(())
}

#[cfg(not(feature = "use_tempfiles"))]
pub fn load_image(stdout: &mut impl Write, id: u32, image: &DynamicImage) -> Result<()> {
    let rgba8 = image.to_rgba8();
    let raw = rgba8.as_raw();
    let encoded = base64::encode(raw);
    let mut iter = encoded.chars().peekable();

    let mut i = 0;
    let chunk_last_index = (encoded.len() as f32 / 4096.0).ceil() as i32 - 1;

    while iter.peek().is_some() {
        let chunk = (&mut iter).take(4096);
        let payload: String = chunk.collect();
        let options =
            if i == 0 {
                format!("f=32,i={id},s={},v={},t=d,m=1", image.width(), image.height())
            } else if i == chunk_last_index {
                "m=1".to_string()
            } else {
                "m=0".to_string()
            };

        write!(stdout, "\x1b_G{options};{payload}\x1b\\")?;

        i += 1;
    }

    writeln!(stdout)?;
    stdout.flush()?;

    Ok(())
}


// blatently stolen from viuer (https://github.com/atanunq/viuer/blob/master/src/printer/kitty.rs)
#[cfg(feature = "use_tempfiles")]
fn store_in_tmp_file(buf: &[u8]) -> Result<std::path::PathBuf> {
    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".mngdx_image")
        .rand_bytes(1)
        .tempfile()?
        // Since the file is persisted, the user is responsible for deleting it afterwards. However,
        // Kitty does this automatically after printing from a temp file.
        .keep()?;

    tmpfile.write_all(buf)?;
    tmpfile.flush()?;
    Ok(path)
}
