use std::{env, sync::Arc};

use anyhow::anyhow;
use axum::{
    Form, Json, RequestPartsExt, Router,
    extract::{FromRequestParts, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
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
    db: Arc<Db>,
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

#[derive(Serialize, Deserialize)]
struct Feed {
    url: String,
    name: String,
    selectors: Selectors,
}

#[derive(Serialize, Deserialize)]
struct Selectors {
    post: Option<String>,
    title: Option<String>,
    link: String,
    description: Option<String>,
    date: Option<String>,
    image: Option<String>,
}

impl From<db::WebNews> for Feed {
    fn from(web_news: db::WebNews) -> Feed {
        Feed {
            url: web_news.url,
            name: web_news.name,
            selectors: Selectors {
                post: web_news.selector_post,
                title: web_news.selector_title,
                link: web_news.selector_link,
                description: web_news.selector_description,
                date: web_news.selector_date,
                image: web_news.selector_image,
            },
        }
    }
}

async fn create_feed(
    State(state): State<AppState>,
    user: User,
    Json(feed): Json<Feed>,
) -> Result<(), AppError> {
    state
        .db
        .create_web_news(&db::WebNews {
            id: -1, // ignored
            url: feed.url,
            name: feed.name,
            owner: user.id,
            selector_post: feed.selectors.post,
            selector_title: feed.selectors.title,
            selector_link: feed.selectors.link,
            selector_description: feed.selectors.description,
            selector_date: feed.selectors.date,
            selector_image: feed.selectors.image,
        })
        .await?;

    Ok(())
}

async fn list_feeds(
    State(state): State<AppState>,
    user: User,
) -> Result<Json<Vec<String>>, AppError> {
    Ok(Json(
        state.db.get_all_web_news_names_by_user_id(user.id).await?,
    ))
}

async fn get_feed(
    State(state): State<AppState>,
    Path(feed): Path<String>,
) -> Result<Json<Feed>, AppError> {
    let Some(web_news) = state.db.get_web_news_by_name(&feed).await? else {
        return Err(anyhow!("no such feed").into());
    };

    Ok(Json(web_news.into()))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    _ = dotenvy::dotenv();

    let db = Arc::new(Db::build(&env::var("DATABASE_URL")?).await?);

    let router = Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/register", post(register_user))
                .route("/login", post(login_user)),
        )
        .route("/feeds", get(list_feeds).post(create_feed))
        .route("/feeds/{name}", get(get_feed))
        .with_state(AppState {
            auth: Auth::new(Arc::clone(&db)),
            db: Arc::clone(&db),
        });

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
