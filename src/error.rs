use chrono::{DateTime, Utc};
use diesel::result::Error as DieselError;
use rocket::http::Status;
use serde::Serialize;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct APIError {
    timestamp: DateTime<Utc>,
    status: Status,
    message: String,
}
impl APIError {
    pub fn new(status: Status, message: String) -> Self {
        APIError {
            timestamp: Utc::now(),
            status,
            message,
        }
    }
}
impl Default for APIError {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            status: Status::InternalServerError,
            message: "Something went wrong processing this request".into(),
        }
    }
}
impl Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendError")
            .field("timestamp", &self.timestamp)
            .field("code", &self.status.code)
            .field("error", &self.status.reason())
            .field("message", &self.message)
            .finish()
    }
}
impl Serialize for APIError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut backend_error = serializer.serialize_struct("BackendError", 4)?;
        backend_error.serialize_field("timestamp", &self.timestamp)?;
        backend_error.serialize_field("code", &self.status.code)?;
        backend_error.serialize_field("error", &self.status.reason())?;
        backend_error.serialize_field("message", &self.message)?;
        backend_error.end()
    }
}
impl Error for APIError {}

impl<'r> rocket::response::Responder<'r, 'static> for APIError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = self.status;
        rocket::Response::build_from(rocket::serde::json::Json(self).respond_to(request)?)
            .status(status)
            .ok()
    }
}

impl From<DieselError> for APIError {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::NotFound => APIError::new(
                Status::NotFound,
                "Something went wrong processing this request.".into(),
            ),
            _ => Default::default(),
        }
    }
}
