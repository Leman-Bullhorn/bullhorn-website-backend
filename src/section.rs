use diesel_derive_enum::DbEnum;
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
