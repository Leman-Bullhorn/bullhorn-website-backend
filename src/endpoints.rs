use crate::article::{ClientArticle, DBArticle, ServerArticle};
use crate::error::APIError;
use crate::writer::{ClientWriter, DBWriter, ServerWriter};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::uri;
use rocket::State;
use rocket::{get, post};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[get("/<files..>", rank = 2)]
pub async fn index(files: PathBuf) -> Option<NamedFile> {
    let path = Path::new("build/").join(files);

    if path.is_dir() {
        NamedFile::open("build/index.html").await.ok()
    } else {
        match NamedFile::open(path).await.ok() {
            Some(file) => Some(file),
            None => NamedFile::open("build/index.html").await.ok(),
        }
    }
}

#[post("/writers", data = "<writer>")]
pub fn post_writers(
    db_connection: &State<Mutex<PgConnection>>,
    writer: Option<Json<ClientWriter<'_>>>,
) -> Result<status::Created<Json<ServerWriter>>, APIError> {
    use crate::schema::writers::dsl::writers;

    let writer = match writer {
        Some(article) => article,
        None => {
            return Err(APIError::new(
                Status::BadRequest,
                "Invalid writer format.".into(),
            ))
        }
    };

    let inserted_writer = diesel::insert_into(writers)
        .values(writer.into_inner())
        .get_results::<DBWriter>(&*db_connection.lock().unwrap())?
        .swap_remove(0);

    let location = uri!("/api", get_writer(inserted_writer.id)).to_string();

    Ok(status::Created::new(location).body(Json(inserted_writer)))
}

#[get("/writers/<id>")]
pub fn get_writer(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
) -> Result<Json<ServerWriter>, APIError> {
    use crate::schema::writers::dsl::{id as writer_id, writers};

    writers
        .filter(writer_id.eq(id))
        .first::<DBWriter>(&*db_connection.lock().unwrap())
        .map(Json)
        .map_err(|err| match err {
            DieselError::NotFound => {
                APIError::new(Status::NotFound, format!("No writer with id {}.", id))
            }
            _ => APIError::from(err),
        })
}

#[post("/articles", data = "<article>")]
pub fn post_articles(
    db_connection: &State<Mutex<PgConnection>>,
    article: Option<Json<ClientArticle<'_>>>,
) -> Result<status::Created<Json<ServerArticle>>, APIError> {
    use crate::schema::*;

    let article = match article {
        Some(article) => article,
        None => {
            return Err(APIError::new(
                Status::BadRequest,
                "Invalid article format.".into(),
            ))
        }
    };

    let db_connection = &*db_connection.lock().unwrap();
    let writer = writers::table
        .filter(writers::id.eq(article.writer_id))
        .first::<DBWriter>(db_connection)
        .map_err(|err| match err {
            DieselError::NotFound => APIError::new(
                Status::NotFound,
                format!("No writer with id {} found.", article.writer_id),
            ),
            _ => APIError::from(err),
        })?;

    let inserted_article = diesel::insert_into(articles::table)
        .values((
            articles::headline.eq(article.headline),
            articles::body.eq(article.body),
            articles::writer_id.eq(article.writer_id),
            articles::publication_date.eq(Utc::now().naive_utc()),
        ))
        .get_results::<DBArticle>(db_connection)?
        .swap_remove(0);

    let ret_article = ServerArticle::new(inserted_article, writer);
    let location = uri!("/api", get_article(ret_article.id)).to_string();

    Ok(status::Created::new(location).body(Json(ret_article)))
}

#[get("/articles")]
pub fn get_articles(
    db_connection: &State<Mutex<PgConnection>>,
) -> Result<Json<Vec<ServerArticle>>, APIError> {
    use crate::schema::articles::dsl::articles;
    use crate::schema::writers::dsl::writers;

    let ret_articles = articles
        .inner_join(writers)
        .load::<(DBArticle, DBWriter)>(&*db_connection.lock().unwrap())?;

    let mut output = Vec::with_capacity(ret_articles.len());

    for (article, writer) in ret_articles {
        output.push(ServerArticle::new(article, writer));
    }

    Ok(Json(output))
}

#[get("/articles/<id>")]
pub fn get_article(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
) -> Result<Json<ServerArticle>, APIError> {
    use crate::schema::articles::dsl::{articles, id as article_id};
    use crate::schema::writers::dsl::writers;

    let ret_article = articles
        .filter(article_id.eq(id))
        .inner_join(writers)
        .first::<(DBArticle, DBWriter)>(&*db_connection.lock().unwrap())
        .map_err(|err| match err {
            DieselError::NotFound => {
                APIError::new(Status::NotFound, format!("No article with id {}.", id))
            }
            _ => APIError::from(err),
        })?;

    Ok(Json(ServerArticle::new(ret_article.0, ret_article.1)))
}

#[get("/<_..>")]
pub fn fallback() -> APIError {
    APIError::new(Status::NotFound, "Invalid endpoint".into())
}
