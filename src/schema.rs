table! {
    use crate::section::SectionMapping;
    use diesel::sql_types::*;

    articles (id) {
        id -> Int4,
        headline -> Varchar,
        slug -> Text,
        body -> Text,
        writer_id -> Int4,
        section -> SectionMapping,
        publication_date -> Timestamptz,
        preview -> Nullable<Text>,
        image_url -> Nullable<Text>,
        drive_file_id -> Nullable<Text>,
    }
}

table! {
    writers (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        bio -> Text,
        title -> Varchar,
    }
}

joinable!(articles -> writers (writer_id));

allow_tables_to_appear_in_same_query!(articles, writers,);
