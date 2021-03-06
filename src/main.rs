#[macro_use]
extern crate diesel;

mod article;
mod auth;
mod endpoints;
mod error;
mod paginated;
mod schema;
mod section;
mod writer;

use diesel::prelude::*;
use rocket::{launch, routes};
use std::env;

#[launch]
fn rocket() -> _ {
    let db_connection = establish_connection();
    let db_connection = std::sync::Mutex::new(db_connection);
    let build_dir = env::var("BUILD_DIR").unwrap_or_else(|_| "build".into());

    rocket::build()
        .mount("/", routes![endpoints::index])
        .mount(
            "/api",
            routes![
                endpoints::get_articles,
                endpoints::get_article,
                endpoints::post_articles,
                endpoints::get_writer,
                endpoints::post_writers,
                endpoints::fallback,
                endpoints::get_writer_by_name,
                endpoints::get_writer_id_articles,
                endpoints::get_sections,
                endpoints::get_section,
                endpoints::post_section,
                endpoints::get_article_by_slug,
                endpoints::login,
                endpoints::current_role,
                endpoints::delete_article,
                endpoints::get_writers
            ],
        )
        .manage(db_connection)
        .manage(build_dir)
}

fn establish_connection() -> PgConnection {
    let _ = dotenv::dotenv();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("error connecting to {}", db_url))
}
