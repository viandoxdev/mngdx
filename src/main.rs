#![allow(dead_code)]

mod api;

use std::time::Duration;

use api::{Api, ApiError};
use uuid::Uuid;

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
    let _ = api.manga_chapters(mid).await.unwrap();
    // TODO: make this easier (put id on objects / make a public interface for this)
    let cid = api
        .cache
        .get_linked(&mid, api::structs::json::data::RelationshipKind::Chapter)
        .unwrap()[0];
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = api.chapter_pages(cid).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let p = api.chapter_pages(cid).await.unwrap();

    println!("{p:?}");

    Ok(())
}
