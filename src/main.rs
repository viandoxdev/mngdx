#![allow(dead_code)]

mod api;

use api::{Api, ApiError};

use crate::api::structs::ChapterAttributes;

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

    //let m = api.manga_view("73965527-b393-4f65-9bc3-2439ec44935a".to_owned()).await.unwrap();
    let c = api.manga_chapters("e78a489b-6632-4d61-b00b-5206f5b8b22b".to_owned()).await.unwrap();
    let chn = c.iter().map(|x| &x.attributes).collect::<Vec<&ChapterAttributes>>();
    //println!("{:?}", api.check_auth().await);
    //println!("{:?}", m);
    //println!("{:?} {}", chn, chn.len());

    Ok(())
}
