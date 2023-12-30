//! Handle errors that may occur while handling a request.
use std::io::Cursor;

use rocket::{Request, response::{self, Responder}, Response, http::{ContentType, Status}};

/// Any error that may occur while handling a request.
#[derive(Debug)]
pub enum Error {
    /// An external service failed or an internal constraint was violated.
    Internal(eyre::Report),
    /// The user could not be authenticated.
    Auth(&'static str),
    /// The user's request was invalid.
    InvalidForm(&'static str),
    /// The requested resource was not found.
    NotFound(&'static str),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, text) = match self {
            Self::Internal(e) => {
                eprintln!("{e:?}");
                (Status::InternalServerError, "internal server error")
            },
            Self::Auth(e) => (rocket::http::Status::Unauthorized, e),
            Self::InvalidForm(e) => (rocket::http::Status::BadRequest, e),
            Self::NotFound(e) => (rocket::http::Status::NotFound, e),
        };
        Response::build()
            .header(ContentType::Plain)
            .status(status)
            .sized_body(text.len(), Cursor::new(text))
            .ok()
    }
}

impl<T: Into<eyre::Report>> From<T> for Error {
    fn from(e: T) -> Self {
        Self::Internal(e.into())
    }
}

impl Error {
    /// Give additional context on what was being attempted if this is an internal error.
    fn wrap_err(self, msg: &'static str) -> Self {
        match self {
            Self::Internal(e) => Self::Internal(e.wrap_err(msg)),
            _ => self,
        }
    }
}

/// Extra methods on `Result` for working with `Error`.
pub trait ResultExt<T> {
    /// Give additional context on what was being attempted.
    fn wrap_err(self, msg: &'static str) -> Result<T, Error>;
}

impl<T, E: Into<Error>> ResultExt<T> for Result<T, E> {
    /// Give additional context on what was being attempted if this is an internal error.
    fn wrap_err(self, msg: &'static str) -> Result<T, Error> {
        self.map_err(|e| e.into().wrap_err(msg))
    }
}
