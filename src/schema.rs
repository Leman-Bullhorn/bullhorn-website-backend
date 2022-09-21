// @generated automatically by Diesel CLI.

diesel::table! {
    articles (id) {
        id -> Int4,
        headline -> Varchar,
        slug -> Text,
        body -> Text,
        writer_id -> Int4,
        section_id -> Int4,
        publication_date -> Timestamptz,
        preview -> Nullable<Text>,
        image_url -> Nullable<Text>,
        drive_link -> Nullable<Text>,
    }
}

diesel::table! {
    sections (id) {
        id -> Int4,
        name -> Text,
        permalink -> Text,
    }
}

diesel::table! {
    writers (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        bio -> Text,
        title -> Varchar,
    }
}

diesel::joinable!(articles -> sections (section_id));
diesel::joinable!(articles -> writers (writer_id));

diesel::allow_tables_to_appear_in_same_query!(
    articles,
    sections,
    writers,
);
