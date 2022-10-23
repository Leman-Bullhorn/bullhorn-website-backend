#![allow(clippy::extra_unused_lifetimes)]
use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize)]
pub struct DBWriter {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub title: String,
    pub bio: Option<String>,
    pub image_url: Option<String>,
}

/// What the client receives when they request a writer.
pub type ServerWriter = DBWriter;

/// What the client sends when they post a writer.
#[derive(Deserialize, Insertable, Debug)]
#[table_name = "writers"]
pub struct ClientWriter<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub title: &'a str,
    pub bio: Option<&'a str>,
    pub image_url: Option<&'a str>,
}
impl<'a> ClientWriter<'a> {
    pub fn new(
        first_name: &'a str,
        last_name: &'a str,
        title: &'a str,
        bio: Option<&'a str>,
        image_url: Option<&'a str>,
    ) -> Self {
        ClientWriter {
            first_name,
            last_name,
            bio,
            title,
            image_url,
        }
    }
}
