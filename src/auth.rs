use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use uuid::Uuid;

use crate::db::{Db, User};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("hashing error")]
    Hash(#[from] bcrypt::BcryptError),
    #[error("no such user")]
    NoUser,
    #[error("wrong password")]
    Password,
    #[error("can't parse the token")]
    ParseToken(#[from] uuid::Error),
    #[error("invalid token")]
    InvalidToken,
    #[error("expired token")]
    ExpiredToken,
}

#[derive(Clone)]
pub struct Auth {
    db: Arc<Db>,
}

impl Auth {
    pub fn new(db: Arc<Db>) -> Auth {
        Auth { db }
    }

    pub async fn register(&self, username: &str, password: String) -> Result<(), AuthError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        self.db.create_user(username, password_hash).await?;
        Ok(())
    }

    pub async fn login(&self, username: &str, password: String) -> Result<String, AuthError> {
        let user = self
            .db
            .get_user_creds_by_name(username)
            .await?
            .ok_or(AuthError::NoUser)?;

        if !bcrypt::verify(password, &user.password_hash)? {
            return Err(AuthError::Password);
        }

        let token = Uuid::new_v4();
        let valid_until = Utc::now() + TimeDelta::days(7);

        self.db.create_session(user.id, token, valid_until).await?;

        Ok(token.to_string())
    }

    pub async fn get_user(&self, token: &str) -> Result<User, AuthError> {
        let session = self
            .db
            .get_session_by_token(token.try_into()?)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if Utc::now() >= session.valid_until {
            return Err(AuthError::ExpiredToken);
        }

        let user = self.db.get_user_by_id(session.owner).await?.unwrap();

        Ok(user)
    }
}
