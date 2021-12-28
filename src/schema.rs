table! {
    articles (id) {
        id -> Int4,
        headline -> Varchar,
        body -> Text,
        writer_id -> Int4,
        publication_date -> Timestamp,
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

allow_tables_to_appear_in_same_query!(
    articles,
    writers,
);
