use std::pin::Pin;

use chrono::{DateTime, Utc};
use futures::Stream;
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

#[derive(Debug)]
pub struct WebNews {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub owner: i32,
    pub selector_post: Option<String>,
    pub selector_title: Option<String>,
    pub selector_link: String,
    pub selector_description: Option<String>,
    pub selector_date: Option<String>,
    pub selector_image: Option<String>,
}

#[derive(Clone)]
pub struct Db(PgPool);

impl Db {
    pub async fn build(url: &str) -> Result<Db, Error> {
        let pool = PgPoolOptions::new().max_connections(8).connect(url).await?;

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

    pub async fn create_user(&self, username: &str, password_hash: String) -> Result<(), Error> {
        query!(
            "INSERT INTO feed_user (name, password_hash) VALUES ($1, $2)",
            username,
            password_hash,
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    pub async fn get_user_creds_by_name(&self, name: &str) -> Result<Option<UserCreds>, Error> {
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

    pub async fn get_session_by_token(&self, token: Uuid) -> Result<Option<Session>, Error> {
        query_as!(
            Session,
            "SELECT owner, token, valid_until FROM user_session WHERE token = $1",
            token,
        )
        .fetch_optional(&self.0)
        .await
    }

    pub async fn create_web_news(&self, web_news: &WebNews) -> Result<(), Error> {
        query!(
            "INSERT INTO web_news (
                url,
                name,
                owner,
                selector_post,
                selector_title,
                selector_link,
                selector_description,
                selector_date,
                selector_image
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            web_news.url,
            web_news.name,
            web_news.owner,
            web_news.selector_post,
            web_news.selector_title,
            web_news.selector_link,
            web_news.selector_description,
            web_news.selector_date,
            web_news.selector_image,
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    pub async fn get_all_web_news_names_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<String>, Error> {
        Ok(
            query!("SELECT name FROM web_news WHERE owner = $1", user_id)
                .fetch_all(&self.0)
                .await?
                .into_iter()
                .map(|r| r.name)
                .collect(),
        )
    }

    pub async fn get_web_news_by_name(&self, name: &str) -> Result<Option<WebNews>, Error> {
        query_as!(WebNews, "SELECT * FROM web_news WHERE name = $1", name)
            .fetch_optional(&self.0)
            .await
    }

    pub async fn delete_web_news_by_name(&self, name: &str) -> Result<(), Error> {
        query!("DELETE FROM web_news WHERE name = $1", name)
            .execute(&self.0)
            .await?;

        Ok(())
    }

    pub async fn update_web_news_by_name(
        &self,
        name: &str,
        web_news: &WebNews,
    ) -> Result<(), Error> {
        query!(
            "UPDATE web_news SET
                url = $2,
                selector_post = $3,
                selector_title = $4,
                selector_link = $5,
                selector_description = $6,
                selector_date = $7,
                selector_image = $8
            WHERE name = $1",
            name,
            web_news.url,
            web_news.selector_post,
            web_news.selector_title,
            web_news.selector_link,
            web_news.selector_description,
            web_news.selector_date,
            web_news.selector_image,
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    pub fn get_all_web_news(&self) -> Pin<Box<dyn Stream<Item = Result<WebNews, sqlx::Error>> + Send + '_>> {
        query_as!(WebNews, "SELECT * FROM web_news").fetch(&self.0)
    }
}
