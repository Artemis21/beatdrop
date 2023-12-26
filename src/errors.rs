//! Handle errors that may occur while handling a request.
use std::io::Cursor;

use rocket::{Request, response::{self, Responder}, Response, http::ContentType};

/// Any error that may occur while handling a request.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error from the database or database driver while making a database query.
    #[error("database error")]
    Database(#[from] sqlx::Error),
    /// An error from the database pool while requesting a connection.
    #[error("database pool error")]
    DatabasePool(#[from] Option<rocket_db_pools::Error<sqlx::Error>>),
    /// An error from the HTTP client while making a request to the Deezer API.
    #[error("outgoing request error")]
    Request(#[from] reqwest::Error),
    /// An error while trying to use the file system.
    #[error("file system error")]
    Io(#[from] std::io::Error),
    /// Some kind of internal constraint was violated.
    #[error("internal error")]
    Internal(&'static str),
    /// The user could not be authenticated.
    #[error("authentication error")]
    Auth(&'static str),
    /// The user's request was invalid.
    #[error("invalid form")]
    InvalidForm(&'static str),
    /// The requested resource was not found.
    #[error("not found")]
    NotFound(&'static str),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        #[cfg(debug_assertions)]
        {
            eprintln!("{self:?}");
        }
        let status = match self {
            Self::Database(_) | Self::DatabasePool(_) | Self::Request(_) | Self::Io(_) | Self::Internal(_) => {
                rocket::http::Status::InternalServerError
            }
            Self::Auth(_) => rocket::http::Status::Unauthorized,
            Self::InvalidForm(_) => rocket::http::Status::BadRequest,
            Self::NotFound(_) => rocket::http::Status::NotFound,
        };
        let body = self.to_string();
        Response::build()
            .header(ContentType::Plain)
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
