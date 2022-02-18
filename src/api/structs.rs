use std::{collections::HashMap, ops::Deref};

use serde::{Serialize, Deserialize};

use super::lang_codes::LanguageCode;

#[derive(Serialize)]
pub struct AuthLoginBody {
    pub username: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct AuthLoginResToken {
    pub session: String,
    pub refresh: String
}
#[derive(Deserialize)]
pub struct AuthLoginRes { pub token: AuthLoginResToken }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCheckRes {
    pub is_authenticated: bool
}

#[derive(Serialize)]
pub struct AuthRefreshBody {
    pub token: String
}

pub type AuthRefreshRes = AuthLoginRes;

#[derive(Deserialize, Debug)]
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
    other: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PublicationDemographic {
    Shounen,
    Shoujo,
    Josei,
    Seinen
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContentRating {
    Safe,
    Suggestive,
    Erotica,
    Pornographic
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MangaState {
    Draft,
    Submitted,
    Published,
    Rejected
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MangaStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled
}

#[derive(Deserialize, Debug)]
pub struct TagAttributes {
    pub name: LocalizedString,
    // NOTE: removed because most of the time empty and breaks stuff
    //pub description: Option<LocalizedString>,
    pub group: String,
    pub version: i32,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: Uuid,
    pub attributes: TagAttributes,
    pub relationships: Vec<Relationship>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MangaAttributes {
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
    pub tags: Vec<Tag>,
    pub state: MangaState,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Debug)]
pub struct Manga {
    pub id: Uuid,
    pub attributes: MangaAttributes,
    pub relationships: Vec<Relationship>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Uuid(String);

impl From<String> for Uuid {
    fn from(v: String) -> Self { Self(v) }
}
impl Deref for Uuid {
    type Target = String;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Deserialize, Debug)]
pub struct LocalizedString(HashMap<LanguageCode, String>);

impl LocalizedString {
    fn get(&self, lang: LanguageCode) -> Option<String> {
        self.0.get(&lang).map(|x| x.clone())
    }
    fn any(&self) -> String {
        self.0.iter().next().map(|x| x.1.clone()).unwrap_or("".to_owned())
    }
    fn get_or_any(&self, lang: LanguageCode) -> String {
        self.get(lang).unwrap_or(self.any())
    }
}
impl From<HashMap<String, String>> for LocalizedString {
    fn from(m: HashMap<String, String>) -> Self {
        Self(m.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

#[derive(Deserialize, Debug)]
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
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Relationship {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub kind: RelationshipKind,
    pub related: Option<RelatedManga>,
}

pub trait Paginate {
    fn total(&self) -> i32;
    fn concat(&mut self, o: Self);
}

#[derive(Deserialize)]
pub struct MangaViewRes {
    pub data: Manga
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChapterAttributes {
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

#[derive(Deserialize, Debug)]
pub struct Chapter {
    pub id: Uuid,
    pub attributes: ChapterAttributes,
    pub relationships: Vec<Relationship>,
}

#[derive(Deserialize)]
pub struct MangaFeedRes {
    pub data: Vec<Chapter>,
    pub total: i32,
}

impl Paginate for MangaFeedRes {
    fn total(&self) -> i32 {
        self.total
    }

    fn concat(&mut self, mut o: Self) {
        self.data.append(&mut o.data);
    }
}
