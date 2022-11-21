use diesel_derive_enum::DbEnum;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    News,
    Opinions,
    Features,
    Science,
    Sports,
    Arts,
    Humor,
}

impl<'r> FromParam<'r> for Section {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        let section = param.to_lowercase();
        match section.as_str() {
            "news" => Ok(Section::News),
            "opinions" => Ok(Section::Opinions),
            "features" => Ok(Section::Features),
            "science" => Ok(Section::Science),
            "sports" => Ok(Section::Sports),
            "arts" => Ok(Section::Arts),
            "humor" => Ok(Section::Humor),
            _ => Err("Invalid section type"),
        }
    }
}
