use std::borrow::Cow;

use crate::writer::DBWriter;
use crate::{schema::articles, section::DBSection};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Associations)]
#[belongs_to(DBWriter, foreign_key = "writer_id")]
#[table_name = "articles"]
pub struct DBArticle {
    pub id: i32,
    pub headline: String,
    pub slug: String,
    pub body: String,
    pub writer_id: i32,
    pub section_id: i32,
    pub publication_date: DateTime<Utc>,
    pub preview: Option<String>,
    pub image_url: Option<String>,
}

/// What the client receives when they request an article.
#[derive(Serialize, Debug)]
pub struct ServerArticle {
    pub id: i32,
    pub headline: String,
    pub slug: String,
    pub body: String,
    pub writer: DBWriter,
    pub section: DBSection,
    pub publication_date: DateTime<Utc>,
    pub preview: String,
    pub image_url: String,
}

impl ServerArticle {
    pub fn new(article: DBArticle, writer: DBWriter, section: DBSection) -> Self {
        ServerArticle {
            id: article.id,
            headline: article.headline,
            slug: article.slug,
            body: article.body,
            writer,
            section,
            publication_date: article.publication_date,
            preview: article.preview.unwrap_or_default(),
            image_url: article.image_url.unwrap_or_default(),
        }
    }
}

/// What the client sends when they post an article.
#[derive(Deserialize, Debug)]
pub struct ClientArticle<'a> {
    pub headline: &'a str,
    #[serde(borrow)]
    pub body: Cow<'a, str>,
    pub writer_id: i32,
    pub section_id: i32,
    pub preview: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
