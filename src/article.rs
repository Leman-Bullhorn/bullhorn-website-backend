use crate::auth::AdminUser;
use crate::error::{APIError, APIResult};
use crate::schema::articles;
use crate::section::Section;
use crate::writer::DBWriter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleContent {
    pub headline: String,
    pub paragraphs: Vec<ArticleParagraph>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleParagraph {
    pub text_alignment: String,
    pub spans: Vec<ArticleSpan>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
//TODO: remove client-side conversion to camelCase and instead use serde options
pub enum SpanContent {
    text {
        content: String,
    },
    anchor {
        href: String,
        content: String,
    },
    image {
        src: String,
        width: String,
        height: String,
        alt: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleSpan {
    pub content: Vec<SpanContent>,
    pub font_style: String,
    pub text_decoration: String,
    pub color: String,
    pub font_weight: String,
}

#[derive(Queryable, Debug, Serialize, Associations, Identifiable)]
#[belongs_to(DBWriter, foreign_key = "writer_id")]
#[table_name = "articles"]
pub struct DBArticle {
    pub id: i32,
    pub headline: String,
    pub focus: String,
    pub slug: String,
    pub body: String,
    pub writer_id: i32,
    pub section: Section,
    pub publication_date: DateTime<Utc>,
    pub image_url: Option<String>,
    pub drive_file_id: Option<String>,
    pub featured: bool,
}

/// What the client receives when they request an article.
#[derive(Serialize, Debug)]
pub struct ServerArticle {
    pub id: i32,
    pub headline: String,
    pub focus: String,
    pub slug: String,
    pub content: ArticleContent,
    pub writer: DBWriter,
    pub section: Section,
    pub publication_date: DateTime<Utc>,
    pub image_url: String,
    pub drive_file_id: Option<String>,
    pub featured: bool,
}

impl ServerArticle {
    pub fn new(article: DBArticle, writer: DBWriter, user: Option<AdminUser>) -> APIResult<Self> {
        let content = serde_json::from_str(&article.body).map_err(|_| APIError::default())?;
        Ok(ServerArticle {
            id: article.id,
            headline: article.headline,
            slug: article.slug,
            content,
            writer,
            section: article.section,
            publication_date: article.publication_date,
            focus: article.focus,
            image_url: article.image_url.unwrap_or_default(),
            drive_file_id: user.and(article.drive_file_id),
            featured: article.featured,
        })
    }

    pub fn with_content(
        article: DBArticle,
        content: ArticleContent,
        writer: DBWriter,
        user: Option<AdminUser>,
    ) -> Self {
        ServerArticle {
            id: article.id,
            headline: article.headline,
            slug: article.slug,
            content,
            writer,
            section: article.section,
            publication_date: article.publication_date,
            focus: article.focus,
            image_url: article.image_url.unwrap_or_default(),
            drive_file_id: user.and(article.drive_file_id),
            featured: article.featured,
        }
    }
}

/// What the client sends when they post an article.
#[derive(Deserialize, Debug)]
pub struct ClientArticle<'a> {
    pub content: ArticleContent,
    pub writer_id: i32,
    pub section: Section,
    pub focus: &'a str,
    pub image_url: Option<&'a str>,
    pub drive_file_id: Option<&'a str>,
    pub featured: Option<bool>,
}
