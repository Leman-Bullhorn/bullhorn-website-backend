use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, DbEnum, Serialize, Deserialize)]
pub enum Section {
    News,
    Opinions,
    Humor,
    Features,
    Science,
    Sports,
    Arts,
}
