//! Manage the database model of a user account.
use super::login_token::Secret;
use crate::DbConn;
use chrono::{DateTime, Utc};
use eyre::{Context, Report, Result};
use serde::Serialize;

/// The database model of a user account.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// The user's unique account ID.
    pub id: i32,
    /// The user's display name.
    pub display_name: Option<String>,
    /// The time this account was created.
    pub created_at: DateTime<Utc>,
    /// The hash of the user's login secret (the login secret forms a part of the login token).
    #[serde(skip)]
    pub secret_hash: Vec<u8>,
}

impl User {
    /// Create a new user account, and return the user's secret login token.
    pub async fn create(db: &mut DbConn) -> Result<(Self, String)> {
        let secret = Secret::new();
        let user = sqlx::query_as!(
            User,
            "INSERT INTO account (secret_hash) VALUES ($1) RETURNING *",
            &secret.hash()
        )
        .fetch_one(db)
        .await
        .wrap_err("error inserting new user account")?;
        let token = secret.into_token(user.id);
        Ok((user, token))
    }

    /// Get a user account by ID.
    ///
    /// See also [`User::from_login`] and [`User::from_claim`].
    pub async fn from_id(db: &mut DbConn, id: i32) -> Result<Option<Self>> {
        sqlx::query_as!(User, "SELECT * FROM account WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .map_err(Report::from)
    }

    /// Set this user's display name.
    ///
    /// Pass `None` to remove the display name.
    pub async fn set_display_name(
        self,
        db: &mut DbConn,
        display_name: Option<&str>,
    ) -> Result<Self> {
        sqlx::query_as!(
            User,
            "UPDATE account SET display_name = $1 WHERE id = $2 RETURNING *",
            display_name.as_deref(),
            self.id
        )
        .fetch_one(db)
        .await
        .map_err(Report::from)
    }

    /// Delete this user account.
    pub async fn delete(self, db: &mut DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM account WHERE id = $1", self.id)
            .execute(db)
            .await
            .map_err(Report::from)
            .map(|_| ())
    }
}
