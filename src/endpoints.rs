use crate::article::{ClientArticle, DBArticle, ServerArticle};
use crate::auth::{create_jwt, AdminUser, LoginInfo, Role, COOKIE_SESSION_TOKEN};
use crate::error::APIError;
use crate::paginated::Paginated;
use crate::section::{ClientSection, DBSection, ServerSection};
use crate::writer::{ClientWriter, DBWriter, ServerWriter};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use rocket::fs::NamedFile;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, uri, State};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new("/[^A-Za-z0-9 -]/g").unwrap();
}

#[get("/<files..>", rank = 10000)]
pub async fn index(build_dir: &State<String>, files: PathBuf) -> Option<NamedFile> {
    let path = Path::new(&**build_dir).join(files);

    async fn open_index(build_path: &str) -> Option<NamedFile> {
        NamedFile::open(Path::new(build_path).join("index.html"))
            .await
            .ok()
    }

    if path.is_dir() {
        open_index(&**build_dir).await
    } else {
        match NamedFile::open(path).await.ok() {
            Some(file) => Some(file),
            None => open_index(&**build_dir).await,
        }
    }
}

#[post("/writers", data = "<writer>")]
pub fn post_writers(
    db_connection: &State<Mutex<PgConnection>>,
    writer: Option<Json<ClientWriter<'_>>>,
    user: Option<AdminUser>,
) -> Result<status::Created<Json<ServerWriter>>, APIError> {
    use crate::schema::writers::dsl::writers;

    user.ok_or_else(APIError::unauthorized)?;

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

#[get("/writers")]
pub fn get_writers(
    db_connection: &State<Mutex<PgConnection>>,
) -> Result<Json<Vec<ServerWriter>>, APIError> {
    use crate::schema::writers::dsl::writers;

    writers
        .load::<DBWriter>(&*db_connection.lock().unwrap())
        .map(Json)
        .map_err(APIError::from)
}

#[get("/writers/<id>", rank = 1)]
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

#[get("/writers/<name>", rank = 2)]
pub fn get_writer_by_name(
    db_connection: &State<Mutex<PgConnection>>,
    name: &str,
) -> Result<Json<ServerWriter>, APIError> {
    use crate::schema::writers::dsl::{first_name, last_name, writers};

    let (query_first_name, query_last_name) = name.split_once('-').ok_or_else(|| {
        APIError::new(
            Status::BadRequest,
            "Name must be in the form \"firstName-LastName\".".into(),
        )
    })?;

    writers
        .filter(first_name.eq(query_first_name))
        .filter(last_name.eq(query_last_name))
        .first::<DBWriter>(&*db_connection.lock().unwrap())
        .map(Json)
        .map_err(|err| match err {
            DieselError::NotFound => {
                APIError::new(Status::NotFound, format!("No writer with name {}.", name))
            }
            _ => APIError::from(err),
        })
}

#[get("/writers/<id>/articles")]
pub fn get_writer_id_articles(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
) -> Result<Json<Vec<ServerArticle>>, APIError> {
    use crate::schema::articles::dsl::{articles, writer_id};
    use crate::schema::sections::dsl::sections;
    use crate::schema::writers::dsl::{id as writer_table_id, writers};

    let db_connection = &*db_connection.lock().unwrap();

    if let Err(err) = writers
        .filter(writer_table_id.eq(id))
        .first::<DBWriter>(db_connection)
    {
        match err {
            DieselError::NotFound => {
                return Err(APIError::new(
                    Status::NotFound,
                    format!("No writer with id {} found.", id),
                ))
            }
            _ => return Err(APIError::from(err)),
        }
    }

    let ret_articles = articles
        .filter(writer_id.eq(id))
        .inner_join(writers)
        .inner_join(sections)
        .load::<(DBArticle, DBWriter, DBSection)>(db_connection)
        .map_err(APIError::from)?;

    Ok(Json(
        ret_articles
            .into_iter()
            .map(|(article, writer, section)| ServerArticle::new(article, writer, section))
            .collect::<Vec<ServerArticle>>(),
    ))
}

#[post("/articles", data = "<article>")]
pub fn post_articles(
    db_connection: &State<Mutex<PgConnection>>,
    article: Option<Json<ClientArticle<'_>>>,
    user: Option<AdminUser>,
) -> Result<status::Created<Json<ServerArticle>>, APIError> {
    use crate::schema::*;

    user.ok_or_else(APIError::unauthorized)?;

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

    let section = sections::table
        .filter(sections::id.eq(article.section_id))
        .first::<DBSection>(db_connection)
        .map_err(|err| match err {
            DieselError::NotFound => APIError::new(
                Status::NotFound,
                format!("No section with id {} found.", article.section_id),
            ),
            _ => APIError::from(err),
        })?;

    let mut slug = article.headline.replace(' ', "-");
    slug.make_ascii_lowercase();
    let slug = SLUG_REGEX.replace_all(&slug, "");

    let inserted_article = diesel::insert_into(articles::table)
        .values((
            articles::headline.eq(article.headline),
            articles::slug.eq(slug),
            articles::body.eq(article.body),
            articles::writer_id.eq(article.writer_id),
            articles::section_id.eq(article.section_id),
            articles::publication_date.eq(Utc::now().naive_utc()),
            articles::preview.eq(article.preview),
            articles::image_url.eq(article.image_url),
        ))
        .get_results::<DBArticle>(db_connection)?
        .swap_remove(0);

    let ret_article = ServerArticle::new(inserted_article, writer, section);
    let location = uri!("/api", get_article(ret_article.id)).to_string();

    Ok(status::Created::new(location).body(Json(ret_article)))
}

#[get("/articles?<limit>&<page>", rank = 2)]
pub fn get_articles(
    db_connection: &State<Mutex<PgConnection>>,
    limit: Option<i64>,
    page: Option<i64>,
) -> Result<Paginated<Vec<ServerArticle>>, APIError> {
    use crate::schema::articles::dsl::{articles, publication_date};
    use crate::schema::sections::dsl::sections;
    use crate::schema::writers::dsl::writers;

    let limit = limit.unwrap_or(10);
    let page = page.unwrap_or(1);
    if page <= 0 {
        return Err(APIError::new(
            Status::BadRequest,
            "Page must be positive".into(),
        ));
    }

    let connection = &*db_connection.lock().unwrap();

    let article_count: i64 = articles.count().get_result(connection)?;

    let ret_articles = articles
        .inner_join(writers)
        .inner_join(sections)
        .order(publication_date.desc())
        .offset((page - 1) * limit)
        .limit(limit)
        .load::<(DBArticle, DBWriter, DBSection)>(connection)?;

    let mut output = Vec::with_capacity(ret_articles.len());

    for (article, writer, section) in ret_articles {
        output.push(ServerArticle::new(article, writer, section));
    }

    Ok(Paginated::new(output, limit, page, article_count))
}

#[delete("/articles/<id>")]
pub fn delete_article(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
    user: Option<AdminUser>,
) -> Result<status::Accepted<()>, APIError> {
    use crate::schema::articles::dsl::{articles, id as article_id};
    use std::cmp::Ordering;

    user.ok_or_else(APIError::unauthorized)?;

    let deleted_count = diesel::delete(articles.filter(article_id.eq(id)))
        .execute(&*db_connection.lock().unwrap())?;

    match deleted_count.cmp(&1) {
        Ordering::Greater => {
            println!("Deleted more than 1 article... something has gone very wrong");
        }
        Ordering::Less => {
            return Err(APIError::new(
                Status::NotFound,
                format!("No article with id {id}"),
            ))
        }
        Ordering::Equal => {}
    }

    Ok(status::Accepted(Some(())))
}

#[get("/articles/<id>", rank = 1)]
pub fn get_article(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
) -> Result<Json<ServerArticle>, APIError> {
    use crate::schema::articles::dsl::{articles, id as article_id};
    use crate::schema::sections::dsl::sections;
    use crate::schema::writers::dsl::writers;

    let ret_article = articles
        .filter(article_id.eq(id))
        .inner_join(writers)
        .inner_join(sections)
        .first::<(DBArticle, DBWriter, DBSection)>(&*db_connection.lock().unwrap())
        .map_err(|err| match err {
            DieselError::NotFound => {
                APIError::new(Status::NotFound, format!("No article with id {}.", id))
            }
            _ => APIError::from(err),
        })?;

    Ok(Json(ServerArticle::new(
        ret_article.0,
        ret_article.1,
        ret_article.2,
    )))
}

#[get("/articles/<slug>", rank = 3)]
pub fn get_article_by_slug(
    db_connection: &State<Mutex<PgConnection>>,
    slug: &str,
) -> Result<Json<ServerArticle>, APIError> {
    use crate::schema::articles::dsl::{articles, slug as article_slug};
    use crate::schema::sections::dsl::sections;
    use crate::schema::writers::dsl::writers;

    let ret_article = articles
        .filter(article_slug.eq(slug))
        .inner_join(writers)
        .inner_join(sections)
        .first::<(DBArticle, DBWriter, DBSection)>(&*db_connection.lock().unwrap())
        .map_err(|err| match err {
            DieselError::NotFound => {
                APIError::new(Status::NotFound, format!("No article with slug {}.", slug))
            }
            _ => APIError::from(err),
        })?;

    Ok(Json(ServerArticle::new(
        ret_article.0,
        ret_article.1,
        ret_article.2,
    )))
}

#[get("/sections")]
pub fn get_sections(
    db_connection: &State<Mutex<PgConnection>>,
) -> Result<Json<Vec<ServerSection>>, APIError> {
    use crate::schema::sections::dsl::sections;

    sections
        .load::<ServerSection>(&*db_connection.lock().unwrap())
        .map_err(APIError::from)
        .map(Json)
}

#[get("/section/<id>")]
pub fn get_section(
    db_connection: &State<Mutex<PgConnection>>,
    id: i32,
) -> Result<Json<ServerSection>, APIError> {
    use crate::schema::sections::dsl::{id as section_id, sections};

    sections
        .filter(section_id.eq(id))
        .first::<ServerSection>(&*db_connection.lock().unwrap())
        .map_err(APIError::from)
        .map(Json)
}

#[post("/sections", data = "<section>")]
pub fn post_section(
    db_connection: &State<Mutex<PgConnection>>,
    section: Option<Json<ClientSection<'_>>>,
    user: Option<AdminUser>,
) -> Result<status::Created<Json<ServerSection>>, APIError> {
    use crate::schema::sections::dsl::sections;

    user.ok_or_else(APIError::unauthorized)?;

    let section = match section {
        Some(section) => section,
        None => {
            return Err(APIError::new(
                Status::BadRequest,
                "Invalid section format.".into(),
            ))
        }
    };

    let inserted_section = diesel::insert_into(sections)
        .values(section.into_inner())
        .get_results::<DBSection>(&*db_connection.lock().unwrap())?
        .swap_remove(0);

    let location = uri!("/api", get_section(inserted_section.id)).to_string();

    Ok(status::Created::new(location).body(Json(inserted_section)))
}

#[post("/login", data = "<login_info>")]
pub fn login(
    jar: &CookieJar<'_>,
    login_info: Option<Json<LoginInfo<'_>>>,
) -> Result<&'static str, APIError> {
    let login_info = match login_info {
        Some(login_info) => login_info,
        None => {
            return Err(APIError::new(
                Status::BadRequest,
                "Missing login information.".into(),
            ))
        }
    };

    let admin_username = std::env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME must be defined");
    let admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be defined");

    if login_info.username == admin_username && login_info.password == admin_password {
        create_jwt(Role::Admin)
            .map(|token| {
                let cookie = Cookie::build(COOKIE_SESSION_TOKEN, token)
                    .secure(false)
                    .http_only(true)
                    .same_site(SameSite::Strict);

                jar.add(cookie.finish());

                "Admin"
            })
            .map_err(|_| APIError::default())
    } else {
        Err(APIError::new(
            Status::Unauthorized,
            "Invalid username or password.".into(),
        ))
    }
}

#[get("/current")]
pub fn current_role(user: Option<AdminUser>) -> &'static str {
    match user {
        Some(_) => "Admin",
        None => "Default",
    }
}

#[get("/<_..>", rank = 9999)]
pub fn fallback() -> APIError {
    APIError::new(Status::NotFound, "Invalid endpoint.".into())
}
