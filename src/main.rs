#![allow(dead_code)]

mod api;

use api::{Api, ApiError};
use uuid::Uuid;

use crate::api::structs::lang_codes::LanguageCode;

#[derive(serde::Deserialize)]
struct TmpConfig {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    pretty_env_logger::init();

    log::info!("Hey !");
    let cfgstr = std::fs::read_to_string("config.json").unwrap();
    let _cfg: TmpConfig = serde_json::from_str(&cfgstr).unwrap();
    let mut api = Api::new();
    //api.login(cfg.username, cfg.password).await?;

    let mid = Uuid::parse_str("73965527-b393-4f65-9bc3-2439ec44935a").unwrap();

    let _ = api.manga_view(mid).await.unwrap();
    let _ = api.manga_view(mid).await.unwrap();
    let _ = api.manga_view(mid).await.unwrap();
    let _ = api.manga_view(mid).await.unwrap();
    let _ = api.manga_chapters(mid).await.unwrap();
    let _ = api.manga_chapters(mid).await.unwrap();
    let _ = api.manga_chapters(mid).await.unwrap();
    let c = api.manga_chapters(mid).await.unwrap();
    let p = api.chapter_pages(c[0]).await.unwrap();
    let d = api.chapter_view(c[0]).await.unwrap();
    let m = api.manga_list(Default::default(), 0, 10).await.unwrap();
    let mut v = vec![];
    for u in m {
        v.push(
            api.manga_view(u)
                .await
                .unwrap()
                .title
                .get_or_any(LanguageCode::English),
        );
    }

    println!("{p:?}");
    println!("{d:?}");
    println!("{v:?}");

    Ok(())
}
