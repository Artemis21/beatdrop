//! Handle errors that may occur while handling a request.
use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};

/// Any error that may occur while handling a request.
#[derive(Debug)]
pub enum ApiError {
    /// An external service failed or an internal constraint was violated.
    Internal(eyre::Report),
    /// The client's request could not be completed because of a client error.
    ///
    /// The status code should be in the 4xx range.
    Client((Status, &'static str)),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, text) = match self {
            Self::Internal(e) => {
                eprintln!("{e:?}");
                (Status::InternalServerError, "internal server error")
            }
            Self::Client((status, text)) => (status, text),
        };
        Response::build()
            .header(ContentType::Plain)
            .status(status)
            .sized_body(text.len(), Cursor::new(text))
            .ok()
    }
}

impl<T: Into<eyre::Report>> From<T> for ApiError {
    fn from(e: T) -> Self {
        Self::Internal(e.into())
    }
}

impl ApiError {
    /// Construct a 400 Bad Request error.
    pub const fn bad_request(msg: &'static str) -> Self {
        Self::Client((Status::BadRequest, msg))
    }

    /// Construct a 401 Unauthorized error.
    ///
    /// NB the function name is spelt in British English, but the status code is American English.
    pub const fn unauthorised(msg: &'static str) -> Self {
        Self::Client((Status::Unauthorized, msg))
    }

    /// Construct a 403 Forbidden error.
    pub const fn forbidden(msg: &'static str) -> Self {
        Self::Client((Status::Forbidden, msg))
    }

    /// Construct a 404 Not Found error.
    pub const fn not_found(msg: &'static str) -> Self {
        Self::Client((Status::NotFound, msg))
    }

    /// Construct a 409 Conflict error.
    pub const fn conflict(msg: &'static str) -> Self {
        Self::Client((Status::Conflict, msg))
    }

    /// Convert this error to a rocket outcome.
    pub const fn into_outcome<S>(self) -> rocket::request::Outcome<S, Self> {
        let status = match self {
            Self::Internal(_) => Status::InternalServerError,
            Self::Client((status, _)) => status,
        };
        rocket::request::Outcome::Error((status, self))
    }
}
