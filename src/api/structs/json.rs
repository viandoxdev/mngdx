/// Body of requests (client -> server)
pub mod body {
    use serde::Serialize;

    // /auth/login
    #[derive(Serialize)]
    pub struct AuthLogin {
        pub username: String,
        pub password: String,
    }

    // /auth/refresh
    #[derive(Serialize)]
    pub struct AuthRefresh {
        pub token: String,
    }
}

/// body or responses (server -> client)
pub mod responses {
    use serde::Deserialize;
    use uuid::Uuid;

    use super::data::{self, Chapter, Manga, Wrapper};

    /// Trait to work with responses with pagination
    pub trait Paginate {
        fn total(&self) -> i32;
        fn concat(&mut self, o: Self);
    }

    // POST /auth/login
    #[derive(Deserialize)]
    pub struct AuthLoginToken {
        pub session: String,
        pub refresh: String,
    }
    #[derive(Deserialize)]
    pub struct AuthLogin {
        pub token: AuthLoginToken,
    }

    // GET /auth/check
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthCheck {
        pub is_authenticated: bool,
    }

    // POST /auth/refresh
    pub type AuthRefresh = AuthLogin;

    // GET /manga/{id}
    #[derive(Deserialize)]
    pub struct MangaView {
        pub data: Wrapper<Manga>,
    }
    // GET /manga/{id}/feed
    #[derive(Deserialize)]
    pub struct MangaFeed {
        pub data: Vec<Wrapper<Chapter>>,
        pub total: i32,
    }

    // GET /at-home/server/{id}
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AtHomeServer {
        pub base_url: String,
        pub chapter: data::AtHomeServerChapter,

        // not actually sent by the api, but necesary to know which chapter this is.
        pub chapter_id: Option<Uuid>,
    }

    // impls

    impl Paginate for MangaFeed {
        fn total(&self) -> i32 {
            self.total
        }
        fn concat(&mut self, mut o: Self) {
            self.data.append(&mut o.data);
        }
    }
}

/// General data, objects as defined by the openapi specs.
pub mod data {
    use serde::Deserialize;
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::api::structs::lang_codes::LanguageCode;

    // Enums

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum PublicationDemographic {
        Shounen,
        Shoujo,
        Josei,
        Seinen,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum ContentRating {
        Safe,
        Suggestive,
        Erotica,
        Pornographic,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum MangaState {
        Draft,
        Submitted,
        Published,
        Rejected,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum MangaStatus {
        Ongoing,
        Completed,
        Hiatus,
        Cancelled,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum RelationshipKind {
        Manga,
        Chapter,
        CoverArt,
        Author,
        Artist,
        ScanlationGroup,
        Tag,
        User,
        CustomList,

        // Custom relationships
        AtHome,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum RelatedManga {
        Monochrome,
        Colored,
        Preserialization,
        Serialization,
        Prequel,
        Sequel,
        MainStory,
        SideStory,
        AdaptedFrom,
        SpinOff,
        BasedOn,
        Doujinshi,
        SameFranchise,
        SharedUniverse,
        AlternateStory,
        AlternateVersion,
    }

    // Structs

    /// Most data fromm the api looks like that (plus a type attribute but we don't need it). This
    /// is needed to hold relationships, the actuall data properties are in the attribute field.
    #[derive(Deserialize, Debug, Clone)]
    pub struct Wrapper<T> {
        pub id: Uuid,
        pub attributes: T,
        pub relationships: Vec<Relationship>,
    }

    /// LocalizedString are a comment occurence in the api, they hold a string in different
    /// languages (not always the sames). This type is here to make working with them easier, as
    /// the underlying type is really just a hashmap.
    #[derive(Deserialize, Debug, Clone)]
    pub struct LocalizedString(HashMap<LanguageCode, String>);

    /// Represents a relationship between data, related is only filled when the kind is Manga.
    #[derive(Deserialize, Debug, Clone)]
    pub struct Relationship {
        pub id: Uuid,
        #[serde(rename = "type")]
        pub kind: RelationshipKind,
        pub related: Option<RelatedManga>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct MangaAttributesLinks {
        #[serde(rename = "al")]
        anilist: Option<String>,
        #[serde(rename = "ap")]
        animeplanet: Option<String>,
        #[serde(rename = "bw")]
        bookwalker: Option<String>,
        #[serde(rename = "mu")]
        mangaupdates: Option<String>,
        #[serde(rename = "nu")]
        novelupdates: Option<String>,
        #[serde(rename = "kt")]
        kitsu: Option<String>,
        #[serde(rename = "amz")]
        amazon: Option<String>,
        #[serde(rename = "ebj")]
        ebookjapan: Option<String>,
        #[serde(rename = "mal")]
        myanimelist: Option<String>,
        #[serde(rename = "cdj")]
        cd_japan: Option<String>,
        raw: Option<String>,
        engtl: Option<String>,
        #[serde(flatten)]
        other: HashMap<String, String>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Tag {
        pub name: LocalizedString,
        // NOTE: removed because most of the time empty and breaks stuff
        //pub description: Option<LocalizedString>,
        pub group: String,
        pub version: i32,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Manga {
        pub title: LocalizedString,
        pub alt_titles: Vec<LocalizedString>,
        pub description: LocalizedString,
        pub is_locked: bool,
        pub links: MangaAttributesLinks,
        pub original_language: String,
        pub last_volume: Option<String>,
        pub last_chapter: Option<String>,
        pub publication_demographic: Option<PublicationDemographic>,
        pub status: Option<MangaStatus>,
        pub year: Option<i32>,
        pub content_rating: ContentRating,
        pub chapter_numbers_reset_on_new_volume: bool,
        pub state: MangaState,
        pub version: i32,
        pub created_at: String,
        pub updated_at: String,
        pub tags: Vec<Wrapper<Tag>>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Chapter {
        pub title: Option<String>,
        pub volume: Option<String>,
        pub chapter: Option<String>,
        pub pages: i32,
        pub translated_language: LanguageCode,
        pub uploader: Option<Uuid>,
        pub external_url: Option<String>,
        pub version: i32,
        pub created_at: String,
        pub updated_at: String,
        pub publish_at: String,
        pub readable_at: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AtHomeServerChapter {
        pub hash: String,
        pub data: Vec<String>,
        pub data_saver: Vec<String>,
    }

    // Impls

    impl LocalizedString {
        fn get(&self, lang: LanguageCode) -> Option<String> {
            self.0.get(&lang).cloned()
        }
        fn any(&self) -> String {
            self.0
                .iter()
                .next()
                .map(|x| x.1.clone())
                .unwrap_or_default()
        }
        fn get_or_any(&self, lang: LanguageCode) -> String {
            self.get(lang).unwrap_or_else(|| self.any())
        }
    }
    impl From<HashMap<String, String>> for LocalizedString {
        fn from(m: HashMap<String, String>) -> Self {
            Self(m.into_iter().map(|(k, v)| (k.into(), v)).collect())
        }
    }

    // impl<T: 'static + Clone> Wrapper<T> {
    //     pub fn cache(self, cache: &mut Graph<Option<Box<dyn Any>>, RelationshipKind>) -> T {
    //         let id = &*self.id;
    //         cache.insert(Some(Box::new(self.attributes.clone())), id.clone());
    //         for r in self.relationships.iter() {
    //             let rid = &*r.id;
    //             if !cache.has(rid.clone()) {
    //                 cache.add(None, rid.clone());
    //             }

    //             let _ = cache.connect(id.clone(), rid.clone(), r.kind.clone());
    //         }

    //         self.attributes
    //     }
    // }
}
