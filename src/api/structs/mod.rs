use std::time::{Duration, Instant};

use self::json::{
    data::{self, Chapter, LocalizedString},
    responses,
};
use super::ApiCache;
use uuid::Uuid;

pub mod json;
pub mod lang_codes;

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
#[derive(Clone)]
pub struct AtHomeServerChapter {
    pub base_url: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
    pub hash: String,
}
// re export types that don't change
pub use data::Tag;

impl Store<Manga> for responses::MangaView {
    fn store(self, cache: &mut ApiCache) -> Manga {
        let m = Manga {
            year: self.data.attributes.year,
            title: self.data.attributes.title,
            links: self.data.attributes.links,
            state: self.data.attributes.state,
            status: self.data.attributes.status,
            version: self.data.attributes.version,
            is_locked: self.data.attributes.is_locked,
            created_at: self.data.attributes.created_at,
            updated_at: self.data.attributes.updated_at,
            alt_titles: self.data.attributes.alt_titles,
            last_volume: self.data.attributes.last_volume,
            description: self.data.attributes.description,
            last_chapter: self.data.attributes.last_chapter,
            content_rating: self.data.attributes.content_rating,
            original_language: self.data.attributes.original_language,
            publication_demographic: self.data.attributes.publication_demographic,
            chapter_numbers_reset_on_new_volume: self
                .data
                .attributes
                .chapter_numbers_reset_on_new_volume,
        };
        cache.insert(self.data.id, m.clone(), None);
        for t in self.data.attributes.tags {
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

impl Store<Vec<Chapter>> for responses::MangaFeed {
    fn store(self, cache: &mut ApiCache) -> Vec<Chapter> {
        let mut res = Vec::with_capacity(self.data.len());
        for c in self.data {
            cache.insert(c.id, c.attributes.clone(), None);
            res.push(c.attributes);
            for r in c.relationships {
                cache.link(&c.id, &r.id, r.kind.clone());
                // link the other way for known kinds
                if r.kind == data::RelationshipKind::Manga {
                    cache.link(&r.id, &c.id, data::RelationshipKind::Chapter)
                }
            }
        }
        res
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
