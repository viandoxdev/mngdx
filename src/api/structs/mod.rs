use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use self::{
    json::{
        data::{self, LocalizedString},
        responses,
    },
    lang_codes::LanguageCode,
};
use super::ApiCache;
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
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut res = Vec::new();

        fn set_simple<T: ToString>(res: &mut Vec<(String, String)>, n: &str, v: T) {
            res.push((n.to_owned(), v.to_string()));
        }

        fn set_option<T: ToString>(res: &mut Vec<(String, String)>, n: &str, o: Option<T>) {
            if let Some(v) = o {
                res.push((n.to_owned(), v.to_string()));
            }
        }

        fn set_vector<T: ToString>(res: &mut Vec<(String, String)>, n: &str, c: &Option<Vec<T>>) {
            if let Some(v) = c {
                for e in v {
                    res.push((format!("{n}[]").to_owned(), e.to_string()));
                }
            }
        }

        fn set_object<K: Display, V: ToString>(
            res: &mut Vec<(String, String)>,
            n: &str,
            o: &HashMap<K, V>,
        ) {
            for (k, v) in o {
                res.push((format!("{n}[{k}]").to_owned(), v.to_string()));
            }
        }

        // dirty replace at the end because https://github.com/seanmonstar/reqwest/issues/530
        set_simple(&mut res, "includedTagsMode", &self.include_tags_mode);
        set_simple(&mut res, "excludedTagsMode", &self.exclude_tags_mode);

        set_option(&mut res, "year", self.year);
        set_option(&mut res, "title", self.title.as_ref());
        set_option(&mut res, "group", self.group);
        set_option(
            &mut res,
            "createdAtSince",
            self.created_at_since.map(|x| x.format("%Y-%m-%dT%H:%M:%S")),
        );
        set_option(
            &mut res,
            "updatedAtSince",
            self.updated_at_since.map(|x| x.format("%Y-%m-%dT%H:%M:%S")),
        );

        set_vector(&mut res, "authors", &self.authors);
        set_vector(&mut res, "artists", &self.artists);
        set_vector(&mut res, "includedTags", &self.include_tags);
        set_vector(&mut res, "excludedTags", &self.exclude_tags);
        set_vector(&mut res, "status", &self.status);
        set_vector(&mut res, "originalLanguage", &self.original_language);
        set_vector(
            &mut res,
            "excludedOriginalLanguage²",
            &self.exclude_original_language,
        );
        set_vector(
            &mut res,
            "availableTranslatedLanguage",
            &self.availible_translated_language,
        );
        set_vector(&mut res, "publicationDemographic", &self.demographic);
        set_vector(&mut res, "ids", &self.ids);
        set_vector(&mut res, "content_rating", &self.content_rating);

        set_object(&mut res, "order", &self.order);

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
// re export types that don't change
pub use data::Chapter;
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
        for r in self.data.relationships {
            cache.link(&self.data.id, &r.id, r.kind);
        }
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
        for r in self.data.relationships {
            cache.link(&self.data.id, &r.id, r.kind.clone());
            cache.link(&r.id, &self.data.id, data::RelationshipKind::Chapter)
        }
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
