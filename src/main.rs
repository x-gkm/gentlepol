use std::env;

use anyhow::anyhow;
use axum::{
    Form, RequestPartsExt, Router,
    extract::{FromRequestParts, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tokio::net::TcpListener;

use crate::auth::{Auth, AuthError};
use crate::db::{Db, User};

mod auth;
mod db;

struct AppError(anyhow::Error);

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
struct AppState {
    auth: Auth,
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

async fn register_user(
    State(state): State<AppState>,
    Form(login_info): Form<LoginInfo>,
) -> Result<(), AppError> {
    state
        .auth
        .register(&login_info.username, login_info.password)
        .await?;

    Ok(())
}

async fn login_user(
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    _ = dotenvy::dotenv();

    let db = Db::build(&env::var("DATABASE_URL")?).await?;

    let router = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .with_state(AppState {
            auth: Auth::new(db),
        });

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
