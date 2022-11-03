table! {
    use crate::section::SectionMapping;
    use diesel::sql_types::*;

    articles (id) {
        id -> Int4,
        headline -> Varchar,
        focus -> Text,
        slug -> Text,
        body -> Text,
        writer_id -> Int4,
        section -> SectionMapping,
        publication_date -> Timestamptz,
        image_url -> Nullable<Text>,
        drive_file_id -> Nullable<Text>,
        featured -> Bool,
    }
}

table! {
    writers (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        title -> Varchar,
        bio -> Nullable<Text>,
        image_url -> Nullable<Text>,
    }
}

joinable!(articles -> writers (writer_id));

allow_tables_to_appear_in_same_query!(articles, writers,);
