use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    Form, Json, RequestPartsExt,
    extract::{FromRequestParts, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use gentlepol::{Feed, db::{Db, User}};
use serde::Deserialize;

use crate::{
    auth::{Auth, AuthError},
    feed::FeedManager,
};

pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> AppError {
        AppError(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub auth: Auth,
    pub feed_manager: FeedManager,
    pub db: Arc<Db>,
}

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

pub async fn register_user(
    State(state): State<AppState>,
    Form(login_info): Form<LoginInfo>,
) -> Result<(), AppError> {
    state
        .auth
        .register(&login_info.username, login_info.password)
        .await?;

    Ok(())
}

pub async fn login_user(
    State(state): State<AppState>,
    Form(login_info): Form<LoginInfo>,
) -> Result<Response, AppError> {
    let result = state
        .auth
        .login(&login_info.username, login_info.password)
        .await;

    match result {
        Ok(token) => Ok(token.to_string().into_response()),
        Err(AuthError::NoUser | AuthError::Password) => {
            Ok((StatusCode::UNAUTHORIZED, "wrong username or password").into_response())
        }
        Err(err) => Err(err.into()),
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<User, Self::Rejection> {
        let jar = parts.extract::<CookieJar>().await?;

        let Some(token) = jar.get("session_id") else {
            return Err(anyhow!("no session_id cookie").into());
        };

        Ok(state.auth.get_user(token.value()).await?)
    }
}

pub async fn create_feed(
    State(state): State<AppState>,
    user: User,
    Json(feed): Json<Feed>,
) -> Result<(), AppError> {
    state.feed_manager.create_feed(user.id, feed).await?;

    Ok(())
}

pub async fn list_feeds(
    State(state): State<AppState>,
    user: User,
) -> Result<Json<Vec<String>>, AppError> {
    Ok(Json(state.feed_manager.feeds_list(user.id).await?))
}

pub async fn get_feed(
    State(state): State<AppState>,
    Path(feed): Path<String>,
    user: User,
) -> Result<Json<Feed>, AppError> {
    Ok(Json(state.feed_manager.feed(user.id, &feed).await?))
}

pub async fn update_feed(
    State(state): State<AppState>,
    Path(feed_name): Path<String>,
    user: User,
    Json(feed): Json<Feed>,
) -> Result<(), AppError> {
    state.feed_manager.update_feed(user.id, &feed_name, feed).await?;
    Ok(())
}

pub async fn delete_feed(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: User,
) -> Result<(), AppError> {
    state.feed_manager.delete_feed(user.id, &name).await?;

    Ok(())
}
