use std::fmt::Display;

use request::*;
use reqwest::Url;
use tokio::time::Duration;
use uuid::Uuid;

use self::structs::*;
use self::{
    cache::ApiCache,
    structs::json::{
        body,
        data::{Chapter, RelationshipKind},
        responses,
    },
};

pub mod cache;
mod request;
pub mod structs;

const API_URL: &str = "https://api.mangadex.org";

#[derive(Debug)]
pub enum ApiError {
    Other,
    BadRequest,
    Auth,
    RateLimit(Duration),
    Request(reqwest::Error),
}
impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Other => write!(f, "Something went wrong with the Api."),
            ApiError::Auth => write!(f, "There was a problem with authenticaton."),
            ApiError::BadRequest => write!(f, "Request is bad, I probably fucked up."),
            ApiError::RateLimit(_) => write!(f, "Too many requests sent."),
            ApiError::Request(_) => {
                write!(f, "There was a (reqwest) error when sending the request.")
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(v: reqwest::Error) -> Self {
        Self::Request(v)
    }
}

impl std::error::Error for ApiError {}

/// Api helper with cache
pub struct Api {
    refresh: Option<String>,
    session: Option<String>,
    api: Url,
    client: reqwest::Client,
    pub cache: ApiCache,
    data_saver: bool,
}

impl Api {
    fn endpoint(&self, path: &str) -> Url {
        let mut copy = self.api.clone();
        copy.set_path(path);

        copy
    }

    pub fn new() -> Self {
        Self {
            refresh: None,
            session: None,
            api: Url::parse(API_URL).unwrap(),
            client: reqwest::Client::new(),
            cache: ApiCache::new(),
            data_saver: false,
        }
    }

    pub fn enable_data_saver(&mut self) {
        self.data_saver = true;
    }
    pub fn toggle_data_saver(&mut self) {
        self.data_saver = !self.data_saver;
    }
    pub fn disable_data_saver(&mut self) {
        self.data_saver = false;
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<(), ApiError> {
        ApiRequest::<body::AuthLogin, responses::AuthLogin, (), _> {
            endpoint: "/auth/login".to_owned(),
            kind: ApiRequestKind::Post,
            body: ApiRequestBody::Json(body::AuthLogin { username, password }),
            done: |api: &mut Api, res: responses::AuthLogin, _: &()| {
                api.refresh = Some(res.token.refresh);
                api.session = Some(res.token.session);
                Ok(())
            },
            ..Default::default()
        }
        .execute_simple(self)
        .await
    }

    pub async fn refresh(&mut self) -> Result<(), ApiError> {
        if let Some(ref refresh) = self.refresh {
            ApiRequest::<body::AuthRefresh, responses::AuthRefresh, (), _> {
                endpoint: "/auth/refresh".to_owned(),
                kind: ApiRequestKind::Post,
                body: ApiRequestBody::Json(body::AuthRefresh {
                    token: refresh.clone(),
                }),
                done: |api: &mut Api, res: responses::AuthRefresh, _: &()| {
                    api.session = Some(res.token.session);
                    Ok(())
                },
                ..Default::default()
            }
            .execute_simple(self)
            .await
        } else {
            Err(ApiError::Auth)
        }
    }

    pub async fn check_auth(&mut self) -> Result<bool, ApiError> {
        if self.session.is_some() {
            return ApiRequest::<(), responses::AuthCheck, bool, _> {
                endpoint: "/auth/check".to_owned(),
                done: |_: &mut Api, res: responses::AuthCheck, _: &()| Ok(res.is_authenticated),
                ..Default::default()
            }
            .execute_simple(self)
            .await;
        }
        Ok(false)
    }

    pub async fn manga_view(&mut self, uuid: Uuid) -> Result<Manga, ApiError> {
        if let Some(cached) = self.cache.get::<Manga>(&uuid) {
            return Ok(cached);
        }

        ApiRequest::<(), responses::MangaView, Manga, _> {
            endpoint: format!("/manga/{}", uuid).to_owned(),
            done: |api: &mut Api, res: responses::MangaView, _: &()| Ok(res.store(&mut api.cache)),
            ..Default::default()
        }
        .execute_simple(self)
        .await
    }

    pub async fn manga_chapters(&mut self, uuid: Uuid) -> Result<Vec<Chapter>, ApiError> {
        if let Some(chapters) = self.cache.get_linked(&uuid, RelationshipKind::Chapter) {
            let cached = chapters
                .iter()
                .filter_map(|id| self.cache.get::<Chapter>(id))
                .collect::<Vec<Chapter>>();

            if cached.len() != chapters.len() {
                log::warn!("Some cached uuid of chapters (from manga) are missing from cache");
            }

            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        ApiRequest::<(), responses::MangaFeed, Vec<Chapter>, ()> {
            endpoint: format!("/manga/{uuid}/feed").to_owned(),
            done: |api, res, _| Ok(res.store(&mut api.cache)),
            ..Default::default()
        }
        .execute_paginate::<500>(self)
        .await
    }

    pub async fn chapter_pages(&mut self, uuid: Uuid) -> Result<Vec<String>, ApiError> {
        let ah = match self
            .cache
            .get_linked(&uuid, RelationshipKind::AtHome)
            .map(|x| x.get(0).map(|x| self.cache.get(x)))
        {
            Some(Some(Some(v))) => v,
            _ => {
                // couldn't find anything in the cache
                ApiRequest::<(), responses::AtHomeServer, AtHomeServerChapter, Uuid> {
                    endpoint: format!("/at-home/server/{uuid}").to_owned(),
                    done: |api, mut res, id| {
                        res.chapter_id = Some(*id);
                        Ok(res.store(&mut api.cache))
                    },
                    info: uuid,
                    ..Default::default()
                }
                .execute(self)
                .await?
            }
        };
        let pages = if self.data_saver {
            ah.data_saver
        } else {
            ah.data
        };
        log::trace!("chapter pages {uuid}");
        Ok(pages
            .into_iter()
            .map(|x| {
                format!(
                    "{}/{}/{}/{x}",
                    ah.base_url,
                    if self.data_saver {
                        "data_saver"
                    } else {
                        "data"
                    },
                    ah.hash
                )
            })
            .collect())
    }

    /// Invalidate cached data of specific uuid, will force the next query (of that object) to
    /// reach out to the api.
    pub fn invalidate_cache(&mut self, uuid: &Uuid) {
        self.cache.remove(uuid);
    }
    /// do the stupid
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
