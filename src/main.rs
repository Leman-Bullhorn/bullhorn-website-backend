#[macro_use]
extern crate diesel;

mod article;
mod endpoints;
mod error;
mod schema;
mod writer;

use diesel::prelude::*;
use rocket::{launch, routes};
use std::env;

#[launch]
fn rocket() -> _ {
    let db_connection = establish_connection();
    let db_connection = std::sync::Mutex::new(db_connection);

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
                endpoints::fallback
            ],
        )
        .manage(db_connection)
}

fn establish_connection() -> PgConnection {
    let _ = dotenv::dotenv();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("error connecting to {}", db_url))
}
