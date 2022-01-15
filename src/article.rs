use crate::schema::articles;
use crate::writer::DBWriter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Associations)]
#[belongs_to(DBWriter, foreign_key = "writer_id")]
#[table_name = "articles"]
pub struct DBArticle {
    pub id: i32,
    pub headline: String,
    pub body: String,
    pub writer_id: i32,
    pub publication_date: DateTime<Utc>,
    pub preview: Option<String>,
    pub image_url: Option<String>,
}

/// What the client receives when they request an article.
#[derive(Serialize, Debug)]
pub struct ServerArticle {
    pub id: i32,
    pub headline: String,
    pub body: String,
    pub writer: DBWriter,
    pub publication_date: DateTime<Utc>,
    pub preview: String,
    pub image_url: String,
}

impl ServerArticle {
    pub fn new(article: DBArticle, writer: DBWriter) -> Self {
        ServerArticle {
            id: article.id,
            headline: article.headline,
            body: article.body,
            writer,
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
    pub body: &'a str,
    pub writer_id: i32,
    pub preview: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
