use crate::{consts::IMAGE_SLOTS, images::ImageManager};

pub struct AppState {
    pub block_name: String,
    pub image_manager: ImageManager<IMAGE_SLOTS>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            block_name: "?".to_owned(),
            image_manager: ImageManager::new(),
        }
    }
}
