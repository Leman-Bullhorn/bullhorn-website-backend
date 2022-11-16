use crate::schema::*;
use crate::section::Section;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize)]
pub struct DBArticleSubmission {
    pub id: i32,
    pub headline: String,
    pub focus: String,
    pub section: Section,
    pub author_id: i32,
    pub drive_file_id: String,
    pub thumbnail_url: Option<String>,
}

/// What the client receives when they request an article submission.
pub type ServerArticleSubmission = DBArticleSubmission;

/// What the client sends when they post a submission.
#[derive(Deserialize, Insertable, Debug)]
#[table_name = "article_submission"]
pub struct ClientArticleSubmission<'a> {
    pub headline: &'a str,
    pub focus: &'a str,
    pub section: Section,
    pub author_id: i32,
    pub drive_file_id: &'a str,
    pub thumbnail_url: Option<&'a str>,
}
impl<'a> ClientArticleSubmission<'a> {
    pub fn new(
        headline: &'a str,
        focus: &'a str,
        section: Section,
        author_id: i32,
        drive_file_id: &'a str,
        thumbnail_url: Option<&'a str>,
    ) -> Self {
        ClientArticleSubmission {
            headline,
            focus,
            section,
            author_id,
            drive_file_id,
            thumbnail_url,
        }
    }
}
