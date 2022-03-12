use std::{
    collections::{HashMap, VecDeque},
    io::Write,
    os::unix::prelude::AsRawFd,
    path::Path,
};

use anyhow::{Error, Result};
use image::DynamicImage;
use reqwest::IntoUrl;
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::consts::IMAGE_SLOTS;

// this is the same as libc::winsize, but I didn't want libc to be exposed (and this implements
// debug)
#[derive(Debug)]
pub struct TermWinSize {
    pub width: u16,
    pub height: u16,
    pub cols: u16,
    pub rows: u16,
}

impl From<libc::winsize> for TermWinSize {
    fn from(v: libc::winsize) -> Self {
        Self {
            width: v.ws_xpixel,
            height: v.ws_ypixel,
            cols: v.ws_col,
            rows: v.ws_row,
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
    if unsafe { libc::ioctl(fd.as_raw_fd(), libc::TIOCGWINSZ, &ws) } != 0 {
        Err(Error::msg("Non zero result for ioctl (TIOCGWINSZ)."))
    } else {
        Ok(ws.into())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ImagePlacement {
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: u32,
    pub y: u32,
    pub z_index: i32,
}

impl Default for ImagePlacement {
    fn default() -> Self {
        Self {
            source_x: 0,
            source_y: 0,
            source_width: None,
            source_height: None,
            width: None,
            height: None,
            x: 0,
            y: 0,
            z_index: 50000,
        }
    }
}

fn display_image(
    stdout: &mut impl Write,
    id: u32,
    pid: u32,
    placement: ImagePlacement,
) -> Result<()> {
    let mut placement_options = String::new();
    let mut add = |opt, name| {
        if let Some(v) = opt {
            placement_options.extend(format!("{name}={v},").chars());
        }
    };

    add(Some(placement.source_x), "x");
    add(Some(placement.source_y), "y");
    add(placement.source_width, "w");
    add(placement.source_height, "h");
    add(placement.width, "c");
    add(placement.height, "r");

    // remove last coma
    placement_options.pop();

    write!(stdout, "\x1b[{};{}H", placement.y, placement.x)?;
    write!(
        stdout,
        "\x1b_Ga=p,C=1,z={},i={id},p={pid},{placement_options};AAAA\x1b\\",
        placement.z_index
    )?;
    stdout.flush()?;
    log::trace!("placement new    {id} {pid}");
    Ok(())
}

fn delete_all_placements(stdout: &mut impl Write) -> Result<()> {
    write!(stdout, "\x1b_Ga=d,d=a;AAAA\x1b\\")?;
    stdout.flush()?;
    log::trace!("placement delete ALL");
    Ok(())
}

fn delete_placement(stdout: &mut impl Write, id: u32, pid: u32) -> Result<()> {
    write!(stdout, "\x1b_Ga=d,d=i,i={id},p={pid};AAAA\x1b\\")?;
    stdout.flush()?;
    log::trace!("placement delete {id} {pid}");
    Ok(())
}

fn unload_image(stdout: &mut impl Write, id: u32) -> Result<()> {
    write!(stdout, "\x1b_Ga=d,d=i,i={id};AAAA\x1b\\")?;
    stdout.flush()?;
    log::trace!("image unload {id}");
    Ok(())
}

#[cfg(feature = "use_tempfiles")]
fn load_image(stdout: &mut impl Write, id: u32, image: &DynamicImage) -> Result<()> {
    let rgba8 = image.to_rgba8();
    let raw = rgba8.as_raw();
    let path = store_in_tmp_file(raw)?;

    write!(
        stdout,
        "\x1b_Gf=32,i={id},s={},v={},t=t;{}\x1b\\",
        image.width(),
        image.height(),
        base64::encode(
            path.to_str()
                .ok_or_else(|| Error::msg("Could not convert path to tempfile to str."))?
        )
    )?;
    writeln!(stdout)?;
    stdout.flush()?;
    log::trace!("image load   {id}");
    Ok(())
}

#[cfg(not(feature = "use_tempfiles"))]
fn load_image(stdout: &mut impl Write, id: u32, image: &DynamicImage) -> Result<()> {
    let rgba8 = image.to_rgba8();
    let raw = rgba8.as_raw();
    let encoded = base64::encode(raw);
    let mut iter = encoded.chars().peekable();

    let mut i = 0;
    let chunk_last_index = (encoded.len() as f32 / 4096.0).ceil() as i32 - 1;

    while iter.peek().is_some() {
        let chunk = (&mut iter).take(4096);
        let payload: String = chunk.collect();
        let options = if i == 0 {
            format!(
                "f=32,i={id},s={},v={},t=d,m=1",
                image.width(),
                image.height()
            )
        } else if i == chunk_last_index {
            "m=0".to_string()
        } else {
            "m=1".to_string()
        };

        write!(stdout, "\x1b_G{options};{payload}\x1b\\")?;
        stdout.flush()?;

        i += 1;
    }
    log::trace!("image load   {id}");
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

/// Struct to help manage and display image in the terminal (only kitty image protocol supported
/// for now).
pub struct ImageManager {
    images: HashMap<u32, Option<DynamicImage>>,
    // Currently loaded images (in the terminal)
    loaded: VecDeque<u32>,
    // Currrent placements (in the terminal)
    state: Vec<(u32, ImagePlacement)>,
    // Placement specified by the user
    requirements: Vec<(u32, ImagePlacement)>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            loaded: VecDeque::with_capacity(IMAGE_SLOTS as usize),
            state: Vec::new(),
            requirements: Vec::new(),
        }
    }

    pub fn add_image(&mut self, id: u32, image: DynamicImage) {
        self.images.insert(id, Some(image));
    }

    pub fn add_from_memory(&mut self, id: u32, bytes: &[u8]) -> Result<()> {
        self.add_image(id, image::load_from_memory(bytes)?);
        Ok(())
    }

    pub fn add_from_file(&mut self, id: u32, path: impl AsRef<Path>) -> Result<()> {
        self.add_image(id, image::io::Reader::open(path)?.decode()?);
        Ok(())
    }

    pub async fn image_from_url(url: impl IntoUrl) -> Result<DynamicImage> {
        let bytes = reqwest::get(url).await?.bytes().await?;
        Ok(image::load_from_memory(&bytes)?)
    }

    pub fn get_image(&self, id: u32) -> Option<&DynamicImage> {
        self.images.get(&id).and_then(|o| o.as_ref())
    }

    pub fn unload_all(&mut self, stdout: &mut impl Write) -> Result<()> {
        for i in self.loaded.drain(..) {
            unload_image(stdout, i)?;
        }
        Ok(())
    }

    /// Remove all images.
    pub fn clear(&mut self) {
        // There is no need to unload any image, as they will get unloaded when necessary on image
        // display.
        self.images.clear();
        self.requirements.clear();
    }

    fn is_loaded(&self, id: u32) -> bool {
        self.loaded.contains(&id)
    }

    fn ensure_loaded(&mut self, stdout: &mut impl Write, id: u32) -> Result<()> {
        if self.get_image(id).is_none() {
            return Err(Error::msg("Image doesn't exist."));
        }

        if !self.is_loaded(id) {
            // NOTE: I know that kitty automatically unloads images when the limit is it. But the
            // limit is up to implementations and leaves no way of knowing when we hit it to know
            // what images are and aren't loaded, which is necessary. Another way would be to parse
            // the response of the escape codes, but that isn't easy as the response is fed back
            // into the input loop.

            // unload image if all slots are full
            if self.loaded.len() == IMAGE_SLOTS as usize {
                let id = self.loaded.back().unwrap();
                log::debug!("unloading {id}");
                unload_image(stdout, *id)?;
                // pop after unload in case unload fails
                self.loaded.pop_back();
            }

            load_image(stdout, id, self.get_image(id).unwrap())?;
            self.loaded.push_front(id);

            Ok(())
        } else {
            // image is already loaded
            Ok(())
        }
    }

    pub fn hide_all_images(&mut self) {
        self.requirements.clear();
    }

    /// Force the next displayed images to be reloaded, this must be used if the terminal unloads
    /// the loaded images or they won't render. (for any reason). Using this with loaded images can
    /// leak images, making them stay loaded and take up memory.
    pub fn force_reload_images(&mut self) {
        self.loaded.clear();
    }

    pub fn draw(&mut self, stdout: &mut impl Write) -> Result<()> {
        let reqs = self.requirements.clone();
        // remove all already satisfied requirements and put them into new_state
        let (mut new_state, reqs): (Vec<(u32, ImagePlacement)>, Vec<(u32, ImagePlacement)>) = reqs.into_iter().partition(|e| self.state.contains(e));

        // display anything that isn't displayed yet
        for r in reqs {
            self.ensure_loaded(stdout, r.0)?;
            display_image(stdout, r.0, 1, r.1.clone())?;
            new_state.push(r);
        }
        
        // hide any placement that was in state but not in new_state
        for s in self.state.iter().filter(|e| !new_state.contains(e)) {
            delete_placement(stdout, s.0, 1)?;
        }

        self.state = new_state;

        Ok(())
    }

    pub fn hide_image(&mut self, id: u32) {
        self.requirements.retain(|e| e.0 != id);
    }

    pub fn display_image(
        &mut self,
        id: u32,
        placement: ImagePlacement,
    ) -> Result<()> {
        if self.get_image(id).is_none() {
            return Err(Error::msg("Image doesn't exist."));
        }

        // if this image already has a placement, overwrite it
        if let Some(index) = self.requirements.iter().position(|e| e.0 == id) {
            self.requirements[index] = (id, placement);
        } else {
            self.requirements.push((id, placement));
        }

        Ok(())
    }

    pub fn display_image_best_fit(
        &mut self,
        id: u32,
        rect: Rect,
        ws: &TermWinSize,
    ) -> Result<()> {
        let width = rect.width as u32;
        let height = rect.height as u32;

        let image = self
            .get_image(id)
            .ok_or_else(|| Error::msg("Image doesn't exist."))?;

        let cell_size = (
            (ws.width as f32) / (ws.cols as f32),
            (ws.height as f32) / (ws.rows as f32),
        );
        let image_aspect = (image.width() as f32) / (image.height() as f32);
        let placement_aspect = (width as f32 * cell_size.0) / (height as f32 * cell_size.1);

        let (pwidth, pheight) = if placement_aspect > image_aspect {
            (
                (height as f32 * cell_size.1 * image_aspect / cell_size.0) as u32,
                height,
            )
        } else {
            (
                width,
                (width as f32 * cell_size.0 / image_aspect / cell_size.1) as u32,
            )
        };

        let offset = ((width - pwidth) / 2, (height - pheight) / 2);
        self.display_image(
            id,
            ImagePlacement {
                width: Some(pwidth),
                height: Some(pheight),
                x: rect.x as u32 + offset.0,
                y: rect.y as u32 + offset.1,
                ..Default::default()
            },
        )?;
        Ok(())
    }
}
