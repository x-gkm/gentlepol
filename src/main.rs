use std::{env, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::{auth::Auth, feed::FeedManager, routes::{AppState, create_feed, delete_feed, get_feed, list_feeds, login_user, register_user, update_feed}};
use gentlepol::db::Db;

mod auth;
mod routes;
mod feed;

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
        .route(
            "/feeds/{name}",
            get(get_feed).put(update_feed).delete(delete_feed),
        )
        .with_state(AppState {
            auth: Auth::new(Arc::clone(&db)),
            feed_manager: FeedManager::new(Arc::clone(&db)),
            db: Arc::clone(&db),
        });

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
