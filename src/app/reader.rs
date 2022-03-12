// Here a reader is a struct that is used to read a chapter (input and render)

use std::io::Write;

use anyhow::Result;
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::images::{ImageManager, TermWinSize};

use super::AppComponents;

pub trait Reader<B: Backend + Write + Send> {
    /// "Advence" the reading (i.e. next page)
    fn next(&mut self);
    /// Next, but the other way
    fn previous(&mut self);
    fn draw(
        &self,
        area: Rect,
        ws: &TermWinSize,
        term: &mut Terminal<B>,
        image_manager: &mut ImageManager,
    ) -> Result<()>;
    /// Start reading a chapter (vec of url to pages)
    fn read(&mut self, chapter: Vec<String>, comps: AppComponents<B>);
}

/// A reader that separates the chapter into distinct pages
pub struct PageReader {
    pages: usize,
    current: usize,
    dirty: bool,
}

impl PageReader {
    pub fn new() -> Self {
        Self {
            pages: 0,
            current: 0,
            dirty: false,
        }
    }
}

impl<B: Backend + Write + Send + 'static> Reader<B> for PageReader {
    fn next(&mut self) {
        self.current = (self.current + 1).min(self.pages - 1);
    }

    fn previous(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    fn draw(
        &self,
        area: Rect,
        ws: &TermWinSize,
        _term: &mut Terminal<B>,
        image_manager: &mut ImageManager,
    ) -> Result<()> {
        let image_id = self.current as u32 + 1;
        image_manager.hide_all_images();
        image_manager.display_image_best_fit(image_id, area, ws)?;
        Ok(())
    }

    fn read(&mut self, chapter: Vec<String>, mut comps: AppComponents<B>) {
        log::debug!("Reader read");
        self.pages = chapter.len();
        self.current = 0;
        for (id, url) in chapter.into_iter().enumerate() {
            let image_manager = comps.image_manager.clone();
            let _ = comps.task_producer.schedule(async move {
                let img = ImageManager::image_from_url(url).await.unwrap();
                log::debug!("Downloaded page");
                image_manager.lock().add_image(id as u32 + 1, img);
                log::debug!("Added page to IM");
            });
        }
    }
}
