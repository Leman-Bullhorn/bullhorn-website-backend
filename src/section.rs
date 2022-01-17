use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize)]
pub struct DBSection {
    pub id: i32,
    pub name: String,
    pub permalink: String,
}

/// What the client receives when they request a section.
pub type ServerSection = DBSection;

/// What the client sends when they post a section.
#[derive(Deserialize, Insertable, Debug)]
#[table_name = "sections"]
pub struct ClientSection<'a> {
    pub name: &'a str,
    pub permalink: &'a str,
}
impl<'a> ClientSection<'a> {
    pub fn new(name: &'a str, permalink: &'a str) -> Self {
        ClientSection { name, permalink }
    }
}
