use chrono::{DateTime, Utc};
use sqlx::{Error, PgPool, postgres::PgPoolOptions, query, query_as};
use uuid::Uuid;

pub struct User {
    pub id: i32,
    pub name: String,
}

pub struct UserCreds {
    pub id: i32,
    pub name: String,
    pub password_hash: String,
}

pub struct Session {
    pub owner: i32,
    pub token: Uuid,
    pub valid_until: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Db(PgPool);

impl Db {
    pub async fn build(url: &str) -> Result<Db, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(url)
            .await?;

        Ok(Db(pool))
    }
    pub async fn create_session(
        &self,
        owner: i32,
        token: Uuid,
        valid_until: DateTime<Utc>,
    ) -> Result<(), Error> {
        query!(
            "INSERT INTO user_session (owner, token, valid_until) VALUES ($1, $2, $3)",
            owner,
            token,
            valid_until,
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: String,
    ) -> Result<(), Error> {
        query!(
            "INSERT INTO feed_user (name, password_hash) VALUES ($1, $2)",
            username,
            password_hash,
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    pub async fn get_user_creds_by_name(
        &self,
        name: &str,
    ) -> Result<Option<UserCreds>, Error> {
        query_as!(
            UserCreds,
            "SELECT id, name, password_hash FROM feed_user WHERE name = $1",
            name,
        )
        .fetch_optional(&self.0)
        .await
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        query_as!(User, "SELECT id, name FROM feed_user WHERE id = $1", id,)
            .fetch_optional(&self.0)
            .await
    }

    pub async fn get_session_by_token(
        &self,
        token: Uuid,
    ) -> Result<Option<Session>, Error> {
        query_as!(
            Session,
            "SELECT owner, token, valid_until FROM user_session WHERE token = $1",
            token,
        )
        .fetch_optional(&self.0)
        .await
    }
}
