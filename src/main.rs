use rocket::fs::NamedFile;
use rocket::{get, launch, routes};
use std::path::{Path, PathBuf};

#[get("/<files..>")]
async fn index(files: PathBuf) -> Option<NamedFile> {
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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
