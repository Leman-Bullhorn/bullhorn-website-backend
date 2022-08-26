pub mod drive_v3_types;

use serde::{Deserialize, Serialize};
use std::path::Path;

use async_google_apis_common as common;
use drive_v3_types as drive;

use drive::FilesService;

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
    let mut files = std::mem::take(&mut resp.files.unwrap());

    while let Some(next_page_token) = &mut resp.next_page_token {
        params.page_token = Some(std::mem::take(next_page_token));
        resp = files_service.list(&params).await?;
        files.extend_from_slice(&resp.files.unwrap());
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
