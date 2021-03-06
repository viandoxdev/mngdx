use std::fmt::Display;

use request::*;
use reqwest::Url;
use tokio::time::Duration;
use uuid::Uuid;

use self::structs::*;
use self::{
    cache::ApiCache,
    structs::json::{body, data::RelationshipKind, responses},
};

pub mod cache;
mod request;
pub mod structs;

const API_URL: &str = "https://api.mangadex.org";
// uuid of the "api object" in the api cache. This object doesn't hold any data itself, but is
// linked to objects (i.e. the tags from the /manga/tag endpoint, as a way to keep them accessible.
const API_UUID: Uuid = Uuid::nil();

#[derive(Debug)]
pub enum ApiError {
    Other,
    BadRequest,
    Auth,
    NotFound,
    Them,
    RateLimit(Duration),
    Request(reqwest::Error),
}
impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Other => write!(f, "Something went wrong with the Api."),
            ApiError::Auth => write!(f, "There was a problem with authenticaton."),
            ApiError::BadRequest => write!(f, "Request is bad, I probably fucked up."),
            ApiError::NotFound => write!(
                f,
                "The requested ressource was not found, either by mangadex, or by us."
            ),
            ApiError::Them => write!(f, "Its not my fault, it's theirs."),
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
        let mut cache = ApiCache::new();
        cache.insert(API_UUID, (), None);
        Self {
            refresh: None,
            session: None,
            api: Url::parse(API_URL).unwrap(),
            client: reqwest::Client::new(),
            cache,
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
        let res = ApiRequest::<body::AuthLogin, responses::AuthLogin> {
            endpoint: "/auth/login".to_owned(),
            kind: ApiRequestKind::Post,
            body: ApiRequestBody::Json(body::AuthLogin { username, password }),
            ..Default::default()
        }
        .send_simple(self)
        .await?;

        self.refresh = Some(res.token.refresh);
        self.session = Some(res.token.session);
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), ApiError> {
        if let Some(ref refresh) = self.refresh {
            let res = ApiRequest::<body::AuthRefresh, responses::AuthRefresh> {
                endpoint: "/auth/refresh".to_owned(),
                kind: ApiRequestKind::Post,
                body: ApiRequestBody::Json(body::AuthRefresh {
                    token: refresh.clone(),
                }),
                ..Default::default()
            }
            .send_simple(self)
            .await?;

            self.session = Some(res.token.session);
            Ok(())
        } else {
            Err(ApiError::Auth)
        }
    }

    pub async fn check_auth(&mut self) -> Result<bool, ApiError> {
        if self.session.is_some() {
            let res = ApiRequest::<(), responses::AuthCheck> {
                endpoint: "/auth/check".to_owned(),
                ..Default::default()
            }
            .send_simple(self)
            .await?;

            Ok(res.is_authenticated)
        } else {
            Ok(false)
        }
    }

    pub async fn tag_view(&mut self, uuid: Uuid) -> Result<Tag, ApiError> {
        // make sure we have all the tags in cache
        self.manga_all_tags().await?;

        self.cache.get::<Tag>(&uuid).ok_or(ApiError::Other)
    }

    pub async fn manga_all_tags(&mut self) -> Result<Vec<Uuid>, ApiError> {
        if let Some(cached) = self.cache.get_linked(&API_UUID, RelationshipKind::Tag) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        let res = ApiRequest::<(), responses::MangaTag> {
            endpoint: "/manga/tag".to_owned(),
            ..Default::default()
        }
        .send(self)
        .await?;

        Ok(res.store(&mut self.cache))
    }

    pub async fn manga_view(&mut self, uuid: Uuid) -> Result<Manga, ApiError> {
        if let Some(cached) = self.cache.get::<Manga>(&uuid) {
            return Ok(cached);
        }

        let res = ApiRequest::<(), responses::MangaView> {
            endpoint: format!("/manga/{uuid}"),
            ..Default::default()
        }
        .send_simple(self)
        .await?;
        Ok(res.store(&mut self.cache))
    }

    pub async fn manga_tags(&mut self, uuid: Uuid) -> Result<Vec<Uuid>, ApiError> {
        // make sure the manga is in the cache
        self.manga_view(uuid).await?;

        if let Some(cached) = self.cache.get_linked(&uuid, RelationshipKind::Tag) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }
        // Something fucked up, we don't know what.
        Err(ApiError::Other)
    }

    /// Search mangas.
    /// WARNING: This always sends a request (caching it would require sending 500+ requests).
    pub async fn manga_list(
        &mut self,
        filter: MangaListFilter,
        offset: i32,
        count: i32,
    ) -> Result<Vec<Uuid>, ApiError> {
        let query = filter.to_query();
        let res = ApiRequest::<(), responses::MangaList> {
            endpoint: "/manga".to_owned(),
            query,
            ..Default::default()
        }
        .send_paginated::<100>(self, offset, count)
        .await?;
        Ok(res.store(&mut self.cache))
    }

    /// WARNING: this always sends a request, as this isn't really cacheable
    pub async fn manga_random(&mut self) -> Result<Uuid, ApiError> {
        let res = ApiRequest::<(), responses::MangaRandom> {
            endpoint: "/manga/random".to_owned(),
            ..Default::default()
        }
        .send(self)
        .await?;

        let id = res.data.id;
        res.store(&mut self.cache);

        Ok(id)
    }

    pub async fn manga_chapters(&mut self, uuid: Uuid) -> Result<Vec<Uuid>, ApiError> {
        if let Some(cached) = self.cache.get_linked(&uuid, RelationshipKind::Chapter) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        let res = ApiRequest::<(), responses::MangaFeed> {
            endpoint: format!("/manga/{uuid}/feed"),
            ..Default::default()
        }
        .send_paginated_all::<500>(self)
        .await?;

        Ok(res.store(&mut self.cache))
    }

    pub async fn manga_volumes(&mut self, uuid: Uuid) -> Result<Vec<Uuid>, ApiError> {
        if let Some(cached) = self.cache.get_linked(&uuid, RelationshipKind::Volume) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        let res = ApiRequest::<(), responses::MangaAggregate> {
            endpoint: format!("/manga/{uuid}/aggregate"),
            ..Default::default()
        }
        .send(self)
        .await?;

        Ok(res.store(&mut self.cache))
    }

    pub fn volume_chapters(&mut self, uuid: Uuid) -> Option<Vec<Uuid>> {
        if let Some(cached) = self.cache.get_linked(&uuid, RelationshipKind::Chapter) {
            if !cached.is_empty() {
                return Some(cached);
            }
        }

        None
    }

    pub fn volume_view(&mut self, uuid: Uuid) -> Option<Volume> {
        self.cache.get::<Volume>(&uuid)
    }

    pub async fn manga_volume(&mut self, name: String, manga: Uuid) -> Result<Uuid, ApiError> {
        let volumes = self.manga_volumes(manga).await?;

        for vid in volumes {
            match self.volume_view(vid) {
                Some(v) if v.volume == name => return Ok(vid),
                _ => {}
            }
        }

        Err(ApiError::NotFound)
    }

    pub async fn chapter_view(&mut self, uuid: Uuid) -> Result<Chapter, ApiError> {
        if let Some(chapter) = self.cache.get(&uuid) {
            return Ok(chapter);
        }

        let res = ApiRequest::<(), responses::ChapterView> {
            endpoint: format!("/chapter/{uuid}").to_owned(),
            ..Default::default()
        }
        .send(self)
        .await?;

        Ok(res.store(&mut self.cache))
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
                let mut res = ApiRequest::<(), responses::AtHomeServer> {
                    endpoint: format!("/at-home/server/{uuid}"),
                    ..Default::default()
                }
                .send(self)
                .await?;
                res.chapter_id = Some(uuid);
                res.store(&mut self.cache)
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
                        "data-saver"
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
