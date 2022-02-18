use std::fmt::Display;

use reqwest::Url;
use request::*;
use structs::*;
use tokio::time::Duration;

pub mod structs;
mod request;
mod lang_codes;

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
            ApiError::Request(_) => write!(f, "There was a (reqwest) error when sending the request."),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(v: reqwest::Error) -> Self {
        Self::Request(v)
    }
}

impl std::error::Error for ApiError {}

pub struct Api {
    refresh: Option<String>,
    session: Option<String>,
    api: Url,
    client: reqwest::Client
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
        }
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<(), ApiError> {
        ApiRequest::<AuthLoginBody, AuthLoginRes, ()> {
            endpoint: "/auth/login".to_owned(),
            kind: ApiRequestKind::Post,
            body: ApiRequestBody::Json(AuthLoginBody {
                username, password
            }),
            done: |api, res| {
                api.refresh = Some(res.token.refresh);
                api.session = Some(res.token.session);
                Ok(())
            },
            ..Default::default()
        }.execute_simple(self).await
    }

    pub async fn refresh(&mut self) -> Result<(), ApiError> {
        if let Some(ref refresh) = self.refresh {
            ApiRequest::<AuthRefreshBody, AuthRefreshRes, ()> {
                endpoint: "/auth/refresh".to_owned(),
                kind: ApiRequestKind::Post,
                body: ApiRequestBody::Json(AuthRefreshBody {
                    token: refresh.clone()
                }),
                done: |api, res| {
                    api.session = Some(res.token.session);
                    Ok(())
                },
                ..Default::default()
            }.execute_simple(self).await
        } else {
            Err(ApiError::Auth)
        }
    }

    pub async fn check_auth(&mut self) -> Result<bool, ApiError> {
        if self.session.is_some() {
            return ApiRequest::<(), AuthCheckRes, bool> {
                endpoint: "/auth/check".to_owned(),
                done: |_, res| {
                    Ok(res.is_authenticated)
                },
                ..Default::default()
            }.execute_simple(self).await;
        }
        Ok(false)
    }
    
    pub async fn manga_view(&mut self, uuid: String) -> Result<structs::Manga, ApiError> {
        ApiRequest::<(), MangaViewRes, Manga> {
            endpoint: format!("/manga/{}", uuid).to_owned(),
            done: |_, res| {
                Ok(res.data)
            },
            ..Default::default()
        }.execute_simple(self).await
    }

    pub async fn manga_chapters(&mut self, uuid: String) -> Result<Vec<structs::Chapter>, ApiError> {
        ApiRequest::<(), MangaFeedRes, Vec<Chapter>> {
            endpoint: format!("/manga/{}/feed", uuid).to_owned(),
            done: |_, res| Ok(res.data),
            ..Default::default()
        }.execute_paginate(self).await
    }
}
