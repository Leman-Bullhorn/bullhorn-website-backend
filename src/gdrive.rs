pub mod drive_v3_types;

use crate::article::{ArticleContent, ArticleParagraph, ArticleSpan, SpanContent};
use anyhow::anyhow;
use async_google_apis_common as common;
use chrono::Datelike;
use drive::FilesService;
use drive_v3_types as drive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read as _, Write};
use std::path::{Path, PathBuf};
use tl::ParserOptions;
use uuid::Uuid;
use zip::result::ZipResult;

pub const DRAFTS_FOLDER_ID: &str = "1BELyMOBd1Orod-Iwn0_Jf7ZHOEydsJb7";
pub const FINALS_FOLDER_ID: &str = "1gDcjDPnt9SU8uM0kAS_H6Ubx0QubjVdw";

fn https_client() -> common::TlsClient {
    let conn = hyper_rustls::HttpsConnector::with_native_roots();
    common::hyper::Client::builder().build(conn)
}

#[derive(Serialize, Deserialize)]
pub struct ServerDriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(rename = "webViewLink")]
    web_view_link: String,
    #[serde(rename = "authorName")]
    author_name: String,
    #[serde(rename = "authorEmail")]
    author_email: String,
    #[serde(rename = "authorPicture")]
    author_picture: String,
}

impl ServerDriveFile {
    fn new(file: drive::File) -> Result<Self, common::Error> {
        let err = || common::Error::msg("An error occurred while fetching Drive data");

        let id = file.id.ok_or_else(err)?;
        let mime_type = file.mime_type.ok_or_else(err)?;
        let name = file.name.ok_or_else(err)?;
        let web_view_link = file.web_view_link.ok_or_else(err)?;
        let file_owner = file.owners.ok_or_else(err)?.pop().ok_or_else(err)?;
        let author_name = file_owner.display_name.ok_or_else(err)?;
        let author_email = file_owner.email_address.ok_or_else(err)?;
        let author_picture = file_owner.photo_link.ok_or_else(err)?;

        Ok(ServerDriveFile {
            id,
            name,
            mime_type,
            web_view_link,
            author_name,
            author_email,
            author_picture,
        })
    }
}

pub async fn make_files_service(client_secret_path: impl AsRef<Path>) -> FilesService {
    let https_client = https_client();

    let service_account_key = common::yup_oauth2::read_service_account_key(client_secret_path)
        .await
        .expect("Client secret could not be read");

    let auth = common::yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key)
        .hyper_client(https_client.clone())
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();

    let scopes = vec![drive::DriveScopes::Drive];
    let mut file_service = drive::FilesService::new(https_client, std::sync::Arc::new(auth));
    file_service.set_scopes(&scopes);
    file_service
}

pub async fn get_files_from_draft_folder(
    files_service: &FilesService,
) -> Result<Vec<ServerDriveFile>, common::Error> {
    let files = get_files_from_folder(files_service, DRAFTS_FOLDER_ID).await?;

    let mut out_files = Vec::with_capacity(files.len());
    for file in files {
        out_files.push(ServerDriveFile::new(file)?);
    }

    Ok(out_files)
}

pub async fn get_files_from_finals_folder(
    files_service: &FilesService,
) -> Result<Vec<ServerDriveFile>, common::Error> {
    let files = get_files_from_folder(files_service, FINALS_FOLDER_ID).await?;

    let mut out_files = Vec::with_capacity(files.len());
    for file in files {
        out_files.push(ServerDriveFile::new(file)?);
    }

    Ok(out_files)
}

pub async fn get_files_from_folder(
    files_service: &FilesService,
    folder_id: impl Into<String>,
) -> Result<Vec<drive::File>, common::Error> {
    let drive_params = drive::DriveParams {
        fields: Some(String::from(
            "files(id, name, mimeType, webViewLink, owners)",
        )),
        ..Default::default()
    };

    let mut params = drive::FilesListParams {
        drive_params: Some(drive_params),
        q: Some(format!("'{}' in parents", folder_id.into())),
        ..Default::default()
    };

    let mut resp = files_service.list(&params).await?;
    let mut files = std::mem::take(&mut resp.files.unwrap_or_default());

    while let Some(next_page_token) = &mut resp.next_page_token {
        params.page_token = Some(std::mem::take(next_page_token));
        resp = files_service.list(&params).await?;
        files.extend_from_slice(&resp.files.unwrap_or_default());
    }

    Ok(files)
}

pub async fn move_file(
    files_service: &FilesService,
    file_id: impl Into<String>,
    dest_folder_id: impl Into<String>,
) -> Result<drive::File, common::Error> {
    let file_id = file_id.into();
    let dest_folder_id = dest_folder_id.into();

    let drive_err = || common::Error::msg("Drive API returned unexpected result");

    let drive_params_get = drive::DriveParams {
        fields: Some("parents".into()),
        ..Default::default()
    };
    let file_get_params = drive::FilesGetParams {
        file_id,
        drive_params: Some(drive_params_get),
        ..Default::default()
    };

    let mut maybe_metadata = files_service.get(&file_get_params).await?;

    // We know that this request will not result in downloading the file so we just supply an empty buf
    let parents = if let common::DownloadResult::Response(file) =
        maybe_metadata.do_it_to_buf(&mut vec![]).await?
    {
        file.parents.ok_or_else(drive_err)?
    } else {
        return Err(drive_err());
    };

    let remove_parents = parents.join(",");

    let drive_params_update = drive::DriveParams {
        fields: Some("id, name, mimeType, webViewLink, owners".into()),
        ..Default::default()
    };
    let file_id = file_get_params.file_id;
    let file_update_params = drive::FilesUpdateParams {
        file_id,
        add_parents: Some(dest_folder_id.clone()),
        remove_parents: Some(remove_parents),
        drive_params: Some(drive_params_update),
        ..Default::default()
    };

    files_service.update(&file_update_params, None).await
}

pub async fn move_file_to_final(
    files_service: &FilesService,
    file_id: impl Into<String>,
) -> Result<ServerDriveFile, common::Error> {
    ServerDriveFile::new(move_file(files_service, file_id, FINALS_FOLDER_ID).await?)
}

pub async fn move_file_to_draft(
    files_service: &FilesService,
    file_id: impl Into<String>,
) -> Result<ServerDriveFile, common::Error> {
    ServerDriveFile::new(move_file(files_service, file_id, DRAFTS_FOLDER_ID).await?)
}

fn get_style_attributes(tag: &tl::HTMLTag) -> HashMap<String, String> {
    let mut map = HashMap::new();

    let styles = tag.attributes().get("style");

    let styles = if let Some(styles) = styles {
        styles
    } else {
        return map;
    };
    let styles = if let Some(styles) = styles {
        styles
    } else {
        return map;
    };

    for style in styles.as_bytes().split(|b| *b == b';') {
        let mut split = style.split(|b| *b == b':');

        match (split.next(), split.next()) {
            (Some(key), Some(value)) => map.insert(
                String::from_utf8_lossy(key).trim().into(),
                String::from_utf8_lossy(value).trim().into(),
            ),
            (_, _) => continue,
        };
    }
    map
}

type ImagePathParts = (i32, u32, String, String);
fn unzip_and_store(zipped_bytes: &[u8]) -> ZipResult<(String, HashMap<String, ImagePathParts>)> {
    let reader = std::io::Cursor::new(zipped_bytes);
    let mut zip = zip::ZipArchive::new(reader)?;

    let mut html_string = String::new();
    let mut file_map = HashMap::new();

    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;
        let file_name = zip_file.name();
        if file_name.starts_with("images/") {
            let mut zip_file_bytes = Vec::with_capacity(zip_file.size() as usize);
            zip_file.read_to_end(&mut zip_file_bytes)?;

            let image_dir = std::env::var("ARTICLE_IMAGE_PATH")
                .expect("environment variable ARTICLE_IMAGE_PATH should be set");

            let extension = std::path::Path::new(zip_file.name())
                .extension()
                .unwrap_or_else(|| "png".as_ref());

            // Using v3 uuid means that there will be no duplicate images stored
            let file_name = Uuid::new_v3(&Uuid::NAMESPACE_URL, &zip_file_bytes).to_string();

            let utc_now = chrono::Utc::now();
            let (year, month) = (utc_now.year(), utc_now.month());

            // image path: images/<year>/<month>/<name>.<extension>
            let mut path = PathBuf::from(image_dir);
            path.push(year.to_string());
            path.push(month.to_string());
            path.push(file_name.clone());
            path.set_extension(extension);

            // Create the image directory if it doesn't exist.
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            let mut dest_file = fs::File::create(&path).unwrap();

            dest_file.write_all(zip_file_bytes.as_slice()).unwrap();
            file_map.insert(
                zip_file.name().to_owned(),
                (
                    year,
                    month,
                    file_name,
                    extension.to_string_lossy().into_owned(),
                ),
            );
        } else if file_name.ends_with(".html") {
            html_string.reserve(zip_file.size() as usize);
            zip_file.read_to_string(&mut html_string).unwrap();
        }
    }

    Ok((html_string, file_map))
}

fn make_a_span(tag: &mut tl::HTMLTag, dom: &tl::VDom) -> SpanContent {
    let href = tag
        .attributes_mut()
        .remove("href")
        .and_then(|it| it)
        .unwrap_or_else(tl::Bytes::new)
        .as_utf8_str()
        .into_owned();

    SpanContent::anchor {
        href,
        content: tag.inner_text(dom.parser()).into_owned(),
    }
}

fn make_image_span(
    tag: &mut tl::HTMLTag,
    image_map: &HashMap<String, ImagePathParts>,
) -> SpanContent {
    let src = tag
        .attributes_mut()
        .remove("src")
        .and_then(|it| it)
        .unwrap_or_else(tl::Bytes::new)
        .as_utf8_str()
        .into_owned();
    let server_src = image_map.get(&src);

    let src = if let Some((year, month, name, extension)) = server_src {
        format!("/image/{year}/{month}/{name}.{extension}")
    } else {
        src
    };

    let alt = tag
        .attributes_mut()
        .remove("alt")
        .and_then(|it| it)
        .unwrap_or_else(tl::Bytes::new)
        .as_utf8_str()
        .into_owned();

    let mut styles = get_style_attributes(tag);

    let width = styles.remove("width").unwrap_or_default();
    let height = styles.remove("height").unwrap_or_default();

    SpanContent::image {
        src,
        alt,
        width,
        height,
    }
}

pub async fn get_article_content(
    files_service: &FilesService,
    file_id: impl Into<String>,
) -> Result<ArticleContent, common::Error> {
    let format_error = || anyhow!("Provided file has invalid format");
    let file_id = file_id.into();
    let file_export_params = drive::FilesExportParams {
        file_id,
        mime_type: "application/zip".into(),
        ..Default::default()
    };

    let mut download = files_service.export(&file_export_params).await?;

    let mut html_bytes = Vec::with_capacity(1024); // 1KB - completely arbitrary
    if let common::DownloadResult::Response(_) = download.do_it_to_buf(&mut html_bytes).await? {
        return Err(common::Error::msg("Not good"));
    }

    let (parsed_string, image_map) = unzip_and_store(&html_bytes)?;

    let dom = tl::parse(&parsed_string, ParserOptions::default())?;

    let mut paragraphs = dom.query_selector("p").ok_or_else(format_error)?;

    let headline = paragraphs
        .next()
        .and_then(|p| p.get(dom.parser()))
        .and_then(|p| p.children())
        .and_then(|children| children.all(dom.parser()).get(0))
        .ok_or_else(format_error)?
        .inner_text(dom.parser())
        .into_owned();

    let mut article_paragraphs = Vec::new();

    for paragraph in paragraphs {
        let paragraph = paragraph
            .get(dom.parser())
            .and_then(|p| p.as_tag())
            .ok_or_else(format_error)?;
        let mut styles = get_style_attributes(paragraph);

        let text_alignment = styles.remove("text-align").unwrap_or_else(|| "left".into());
        let text_indent = styles.remove("text-indent").unwrap_or_else(|| "0".into());
        let margin_left = styles.remove("margin-left").unwrap_or_else(|| "0".into());
        let margin_right = styles.remove("margin-right").unwrap_or_else(|| "0".into());

        let mut article_spans = Vec::new();
        let spans = paragraph
            .query_selector(dom.parser(), "span")
            .ok_or_else(format_error)?;
        for span in spans {
            let span = span
                .get(dom.parser())
                .and_then(|s| s.as_tag())
                .ok_or_else(format_error)?;
            let mut styles = get_style_attributes(span);

            let mut content = Vec::new();
            for child in span.children().top().iter() {
                let mut node = child.get(dom.parser()).ok_or_else(format_error)?.clone();
                match &mut node {
                    tl::Node::Tag(tag) if tag.name() == "a" => {
                        let a = make_a_span(tag, &dom);
                        content.push(a);
                    }
                    tl::Node::Tag(tag) if tag.name() == "img" => {
                        let image = make_image_span(tag, &image_map);
                        content.push(image);
                    }
                    tl::Node::Raw(text) => {
                        content.push(SpanContent::text {
                            content: text.as_utf8_str().into_owned(),
                        });
                    }
                    _ => {}
                }
            }

            let font_style = styles
                .remove("font-style")
                .unwrap_or_else(|| "normal".into());
            let text_decoration = styles
                .remove("text-decoration")
                .unwrap_or_else(|| "none".into());
            let color = styles.remove("color").unwrap_or_else(|| "#000000".into());
            let font_weight = styles.remove("font-weight").unwrap_or_else(|| "400".into());

            let article_span = ArticleSpan {
                content,
                font_style,
                text_decoration,
                color,
                font_weight,
            };

            article_spans.push(article_span);
        }

        let article_paragraph = ArticleParagraph {
            margin_left,
            margin_right,
            text_alignment,
            text_indent,
            spans: article_spans,
        };
        article_paragraphs.push(article_paragraph);
    }

    let article_content = ArticleContent {
        headline,
        paragraphs: article_paragraphs,
    };

    Ok(article_content)
}
