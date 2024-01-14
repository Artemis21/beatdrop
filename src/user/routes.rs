//! API routes and request guards for user accounts and sessions.
use crate::{track, ApiError, Connection, DbConn, Session, Transaction, User};
use rocket::{
    delete, get, patch, post,
    request::{self, FromRequest},
    routes,
    serde::json::Json,
    Request,
};

use serde::{Deserialize, Serialize};

/// Collect API routes for user accounts and sessions.
pub fn routes() -> Vec<rocket::Route> {
    routes![new_user, get_user, update_user, delete_user, login_secret,]
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(token) = req.headers().get_one("Authorization") else {
            return ApiError::unauthorised("auth header missing").into_outcome();
        };
        match Self::from_auth_header(token) {
            Ok(claim) => request::Outcome::Success(claim),
            Err(e) => ApiError::unauthorised(e).into_outcome(),
        }
    }
}

impl Session {
    /// Get the authenticated user's account.
    pub async fn user(self, db: &mut DbConn) -> Result<User, ApiError> {
        User::from_session(self, db)
            .await?
            .ok_or_else(|| ApiError::unauthorised("invalid session token"))
    }
}

/// JSON response to a track search query.
#[derive(Default, Serialize)]
struct SearchResults {
    /// The tracks that were found.
    tracks: Vec<track::Meta>,
}
/// The response body for a newly created user.
#[derive(Serialize)]
struct NewUser {
    /// The secret login token for the new user.
    login: String,
}

/// Create a new user.
#[post("/users/me")]
async fn new_user(mut conn: Connection) -> Result<Json<NewUser>, ApiError> {
    let (_user, secret) = User::create(&mut conn).await?;
    Ok(Json(NewUser { login: secret }))
}

/// Get information on the authenticated user's account.
#[get("/users/me")]
async fn get_user(mut conn: Connection, auth: Session) -> Result<Json<User>, ApiError> {
    Ok(Json(auth.user(&mut conn).await?))
}

/// The request body for updating a user account.
#[derive(Deserialize)]
struct UpdateUser {
    /// The new display name for the user, or `null` to keep the current one.
    display_name: Option<String>,
}

/// Update the authenticated user's account.
#[patch("/users/me", data = "<body>")]
async fn update_user(
    mut tx: Transaction<'_>,
    auth: Session,
    body: Json<UpdateUser>,
) -> Result<Json<User>, ApiError> {
    let mut user = auth.user(&mut tx).await?;
    if let Some(display_name) = &body.display_name {
        user = user.set_display_name(&mut tx, Some(display_name)).await?;
    }
    tx.commit().await?;
    Ok(Json(user))
}

/// Delete the authenticated user's account.
#[delete("/users/me")]
async fn delete_user(mut tx: Transaction<'_>, auth: Session) -> Result<(), ApiError> {
    let user = auth.user(&mut tx).await?;
    user.delete(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

/// The request body for logging in.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "method")]
enum Login {
    /// Log in using a login secret. Currently the only supported method.
    Secret {
        /// The secret login token.
        secret: String,
    },
}

/// The response body for a newly created session.
#[derive(Serialize)]
struct SessionResponse {
    /// The session token.
    session: String,
}

/// Create a new session using a secret login token.
#[post("/sessions", data = "<body>")]
async fn login_secret(
    mut conn: Connection,
    body: Json<Login>,
) -> Result<Json<SessionResponse>, ApiError> {
    let user = match &*body {
        Login::Secret { secret } => match User::from_login_token(&mut conn, secret).await? {
            Some(user) => user,
            _ => return Err(ApiError::unauthorised("invalid login secret")),
        },
    };
    let session = user.session_token();
    Ok(Json(SessionResponse { session }))
}
