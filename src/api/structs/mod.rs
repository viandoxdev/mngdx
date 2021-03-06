use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use crate::api::request::ApiRequestQuery;

use self::{
    json::{
        data::{self, LocalizedString},
        responses,
    },
    lang_codes::LanguageCode,
};
use super::{ApiCache, API_UUID};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;
pub mod json;
pub mod lang_codes;

pub enum IncludeMode {
    And,
    Or,
}
impl Display for IncludeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MangaListOrder {
    Asc,
    Desc,
}

#[derive(Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MangaListOrderCriteria {
    Title,
    Year,
    CreatedAt,
    UpdatedAt,
    LatestUploadedChapter,
    FollowedCount,
    Relevance,
}

pub struct MangaListFilter {
    title: Option<String>,
    authors: Option<Vec<Uuid>>,
    artists: Option<Vec<Uuid>>,
    year: Option<i32>,
    include_tags: Option<Vec<Uuid>>,
    include_tags_mode: IncludeMode,
    exclude_tags: Option<Vec<Uuid>>,
    exclude_tags_mode: IncludeMode,
    status: Option<Vec<data::MangaStatus>>,
    original_language: Option<Vec<LanguageCode>>,
    exclude_original_language: Option<Vec<LanguageCode>>,
    availible_translated_language: Option<Vec<LanguageCode>>,
    demographic: Option<Vec<data::PublicationDemographic>>,
    ids: Option<Vec<Uuid>>,
    content_rating: Option<Vec<data::ContentRating>>,
    created_at_since: Option<DateTime<Utc>>,
    updated_at_since: Option<DateTime<Utc>>,
    group: Option<Uuid>,
    order: HashMap<MangaListOrderCriteria, MangaListOrder>,
}

impl Default for MangaListFilter {
    fn default() -> Self {
        let mut order = HashMap::new();
        order.insert(
            MangaListOrderCriteria::LatestUploadedChapter,
            MangaListOrder::Desc,
        );
        Self {
            ids: None,
            year: None,
            title: None,
            group: None,
            order,
            status: None,
            authors: None,
            artists: None,
            demographic: None,
            include_tags: None,
            exclude_tags: None,
            content_rating: None,
            created_at_since: None,
            updated_at_since: None,
            include_tags_mode: IncludeMode::And,
            exclude_tags_mode: IncludeMode::Or,
            original_language: None,
            exclude_original_language: None,
            availible_translated_language: None,
        }
    }
}

impl Display for data::MangaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            enquote::unquote(&serde_json::to_string(self).unwrap()).unwrap()
        )
    }
}

impl Display for data::PublicationDemographic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            enquote::unquote(&serde_json::to_string(self).unwrap()).unwrap()
        )
    }
}

impl Display for data::ContentRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            enquote::unquote(&serde_json::to_string(self).unwrap()).unwrap()
        )
    }
}

impl Display for MangaListOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            enquote::unquote(&serde_json::to_string(self).unwrap()).unwrap()
        )
    }
}

impl Display for MangaListOrderCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            enquote::unquote(&serde_json::to_string(self).unwrap()).unwrap()
        )
    }
}

impl MangaListFilter {
    pub fn to_query(&self) -> ApiRequestQuery {
        let mut res = ApiRequestQuery::new();

        res.insert("includedTagsMode", &self.include_tags_mode);
        res.insert("excludedTagsMode", &self.exclude_tags_mode);
        res.insert_map("order", &self.order);
        res.insert_option("year", self.year);
        res.insert_option("title", self.title.as_ref());
        res.insert_option("group", self.group);
        res.insert_option(
            "createdAtSince",
            self.created_at_since.map(|x| x.format("%Y-%m-%dT%H:%M:%S")),
        );
        res.insert_option(
            "updatedAtSince",
            self.updated_at_since.map(|x| x.format("%Y-%m-%dT%H:%M:%S")),
        );
        res.insert_vec_option("authors", &self.authors);
        res.insert_vec_option("artists", &self.artists);
        res.insert_vec_option("includedTags", &self.include_tags);
        res.insert_vec_option("excludedTags", &self.exclude_tags);
        res.insert_vec_option("status", &self.status);
        res.insert_vec_option("originalLanguage", &self.original_language);
        res.insert_vec_option("excludedOriginalLanguage", &self.exclude_original_language);
        res.insert_vec_option(
            "availableTranslatedLanguage",
            &self.availible_translated_language,
        );
        res.insert_vec_option("publicationDemographic", &self.demographic);
        res.insert_vec_option("ids", &self.ids);
        res.insert_vec_option("content_rating", &self.content_rating);
        res
    }
}

/// Trait used to store data gotten from the api (json::responses) into an api cache and return the
/// obtained object.
pub trait Store<T> {
    fn store(self, cache: &mut ApiCache) -> T;
}

#[derive(Clone, Debug)]
pub struct Manga {
    pub title: LocalizedString,
    pub alt_titles: Vec<data::LocalizedString>,
    pub description: data::LocalizedString,
    pub is_locked: bool,
    pub links: data::MangaAttributesLinks,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<data::PublicationDemographic>,
    pub status: Option<data::MangaStatus>,
    pub year: Option<i32>,
    pub content_rating: data::ContentRating,
    pub chapter_numbers_reset_on_new_volume: bool,
    pub state: data::MangaState,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}
impl From<data::Manga> for Manga {
    fn from(v: data::Manga) -> Self {
        Manga {
            year: v.year,
            title: v.title,
            links: v.links,
            state: v.state,
            status: v.status,
            version: v.version,
            is_locked: v.is_locked,
            created_at: v.created_at,
            updated_at: v.updated_at,
            alt_titles: v.alt_titles,
            last_volume: v.last_volume,
            description: v.description,
            last_chapter: v.last_chapter,
            content_rating: v.content_rating,
            original_language: v.original_language,
            publication_demographic: v.publication_demographic,
            chapter_numbers_reset_on_new_volume: v.chapter_numbers_reset_on_new_volume,
        }
    }
}
#[derive(Clone)]
pub struct AtHomeServerChapter {
    pub base_url: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
    pub hash: String,
}

#[derive(Clone)]
pub struct Volume {
    pub volume: String,
}

// re export types that don't change
pub use data::Chapter;
pub use data::CoverArt;
pub use data::Tag;

impl Store<Manga> for responses::MangaView {
    fn store(self, cache: &mut ApiCache) -> Manga {
        let tags = self.data.attributes.tags.clone();
        let m: Manga = self.data.attributes.into();

        cache.insert(self.data.id, m.clone(), None);
        for t in tags {
            cache.insert(t.id, t.attributes, None);
            cache.link(&self.data.id, &t.id, data::RelationshipKind::Tag);
            cache.link(&t.id, &self.data.id, data::RelationshipKind::Manga);
        }
        store_relationships(
            cache,
            self.data.relationships,
            self.data.id,
            data::RelationshipKind::Manga,
        );
        m
    }
}

impl Store<Vec<Uuid>> for responses::MangaList {
    fn store(self, cache: &mut ApiCache) -> Vec<Uuid> {
        let mut res = Vec::with_capacity(self.data.len());
        for m in self.data {
            res.push(m.id);
            responses::MangaView { data: m }.store(cache);
        }
        res
    }
}

impl Store<Vec<Uuid>> for responses::MangaFeed {
    fn store(self, cache: &mut ApiCache) -> Vec<Uuid> {
        let mut res = Vec::with_capacity(self.data.len());
        for c in self.data {
            res.push(c.id);
            responses::ChapterView { data: c }.store(cache);
        }
        res
    }
}

impl Store<Chapter> for responses::ChapterView {
    fn store(self, cache: &mut ApiCache) -> Chapter {
        cache.insert(self.data.id, self.data.attributes.clone(), None);
        store_relationships(
            cache,
            self.data.relationships,
            self.data.id,
            data::RelationshipKind::Chapter,
        );
        self.data.attributes
    }
}

impl Store<AtHomeServerChapter> for responses::AtHomeServer {
    fn store(self, cache: &mut ApiCache) -> AtHomeServerChapter {
        // create new uuid as this isn't a mangadex object, but one we create to represent data
        // that we need to cache
        let id = Uuid::new_v4();
        let m = AtHomeServerChapter {
            base_url: self.base_url,
            data: self.chapter.data,
            data_saver: self.chapter.data_saver,
            hash: self.chapter.hash,
        };
        let cid = self.chapter_id.unwrap();
        cache.insert(
            id,
            m.clone(),
            Some(Instant::now() + Duration::from_secs(900)),
        );
        cache.link(&cid, &id, data::RelationshipKind::AtHome);
        cache.link(&id, &cid, data::RelationshipKind::Chapter);
        m
    }
}

impl Store<Vec<Uuid>> for responses::MangaAggregate {
    fn store(self, cache: &mut ApiCache) -> Vec<Uuid> {
        let mut res = Vec::with_capacity(self.volumes.len());

        for (k, e) in self.volumes {
            let id = Uuid::new_v4();
            let v = Volume { volume: k };

            res.push(id);

            cache.insert(id, v, None);

            // link volume to manga
            cache.link(&self.manga_id.unwrap(), &id, data::RelationshipKind::Volume);
            cache.link(&id, &self.manga_id.unwrap(), data::RelationshipKind::Manga);

            for c in e.chapters.into_values() {
                for cid in std::iter::once(c.id).chain(c.others.into_iter()) {
                    // link chapters to volume
                    cache.link(&cid, &id, data::RelationshipKind::Volume);
                    cache.link(&id, &cid, data::RelationshipKind::Chapter);
                }
            }
        }
        res
    }
}

impl Store<Vec<Uuid>> for responses::MangaTag {
    fn store(self, cache: &mut ApiCache) -> Vec<Uuid> {
        let mut res = Vec::with_capacity(self.data.len());

        for e in self.data {
            cache.insert(e.id, e.attributes, None);
            cache.link(&API_UUID, &e.id, data::RelationshipKind::Tag);

            res.push(e.id);
        }
        res
    }
}

impl Store<Vec<Uuid>> for responses::CoverArtList {
    fn store(self, cache: &mut ApiCache) -> Vec<Uuid> {
        let mut res = Vec::with_capacity(self.data.len());
        for c in self.data {
            res.push(c.id);
            responses::CoverArt { data: c }.store(cache);
        }
        res
    }
}

impl Store<CoverArt> for responses::CoverArt {
    fn store(self, cache: &mut ApiCache) -> CoverArt {
        cache.insert(self.data.id, self.data.attributes.clone(), None);
        store_relationships(
            cache,
            self.data.relationships,
            self.data.id,
            data::RelationshipKind::CoverArt,
        );
        self.data.attributes
    }
}

impl Store<CoverArt> for responses::MangaCoverArt {
    fn store(self, cache: &mut ApiCache) -> CoverArt {
        // set main cover art relationship where necessary
        for c in &self.data.relationships {
            if c.kind == data::RelationshipKind::Manga {
                cache.link(&c.id, &self.data.id, data::RelationshipKind::MainCoverArt);
                break;
            }
        }

        responses::CoverArt { data: self.data }.store(cache)
    }
}

fn store_relationships(
    cache: &mut ApiCache,
    relationships: Vec<data::Relationship>,
    uuid: Uuid,
    kind: data::RelationshipKind,
) {
    for r in relationships {
        cache.link(&uuid, &r.id, r.kind);
        cache.link(&r.id, &uuid, kind.clone());
    }
}
