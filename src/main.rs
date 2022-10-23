#[macro_use]
extern crate diesel;

mod article;
mod auth;
mod endpoints;
mod error;
mod gdrive;
mod paginated;
mod schema;
mod section;
mod writer;

use diesel::prelude::*;
use rocket::fs::FileServer;
use rocket::{launch, routes};
use std::env;
use std::sync::Mutex;

#[launch]
async fn rocket() -> _ {
    let db_connection = Mutex::new(establish_connection());
    let build_dir = env::var("BUILD_DIR").unwrap_or_else(|_| "build".into());

    let client_secret_path = env::var("CLIENT_SECRET_PATH")
        .expect("environment variable 'CLIENT_SECRET_PATH' should be set");

    let images_dir = env::var("ARTICLE_IMAGE_PATH")
        .expect("environment variable 'ARTICLE_IMAGE_PATH' should be set");

    let file_service = gdrive::make_files_service(client_secret_path).await;

    rocket::build()
        .mount("/", routes![endpoints::index])
        .mount(
            "/api",
            routes![
                endpoints::get_articles,
                endpoints::get_article,
                endpoints::post_articles,
                endpoints::patch_article_by_id,
                endpoints::get_writer,
                endpoints::post_headshot,
                endpoints::post_writers,
                endpoints::patch_writer_by_id,
                endpoints::api_fallback,
                endpoints::get_writer_by_name,
                endpoints::get_writer_id_articles,
                endpoints::get_article_by_slug,
                endpoints::login,
                endpoints::current_role,
                endpoints::delete_article,
                endpoints::get_writers,
                endpoints::get_drive_drafts,
                endpoints::get_drive_finals,
                endpoints::move_draft_to_final,
                endpoints::move_final_to_draft,
                endpoints::get_file_content,
            ],
        )
        .mount("/image", routes![endpoints::image_fallback])
        .mount("/image", FileServer::from(images_dir))
        .manage(db_connection)
        .manage(build_dir)
        .manage(file_service)
}

fn establish_connection() -> PgConnection {
    _ = dotenvy::dotenv();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("error connecting to {}", db_url))
}
