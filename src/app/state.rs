pub struct AppState {
    pub block_name: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            block_name: "?".to_owned(),
        }
    }
}
