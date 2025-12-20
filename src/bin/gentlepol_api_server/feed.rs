use std::sync::Arc;

use serde::{Deserialize, Serialize};

use gentlepol::db::{Db, WebNews};

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("no such feed")]
    NoFeed,
    #[error("can't access")]
    Access,
}

#[derive(Clone)]
pub struct FeedManager {
    db: Arc<Db>,
}

#[derive(Serialize, Deserialize)]
pub struct Feed {
    pub url: String,
    pub name: String,
    pub selectors: Selectors,
}

#[derive(Serialize, Deserialize)]
pub struct Selectors {
    pub post: Option<String>,
    pub title: Option<String>,
    pub link: String,
    pub description: Option<String>,
    pub date: Option<String>,
    pub image: Option<String>,
}

impl From<WebNews> for Feed {
    fn from(web_news: WebNews) -> Feed {
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

impl FeedManager {
    pub fn new(db: Arc<Db>) -> FeedManager {
        FeedManager { db }
    }

    pub async fn create_feed(&self, user_id: i32, feed: Feed) -> Result<(), FeedError> {
        self.db
            .create_web_news(&WebNews {
                id: -1, // ignored

                url: feed.url,
                name: feed.name,
                owner: user_id,
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

    pub async fn feeds_list(&self, user_id: i32) -> Result<Vec<String>, FeedError> {
        Ok(self.db.get_all_web_news_names_by_user_id(user_id).await?)
    }

    pub async fn feed(&self, user_id: i32, name: &str) -> Result<Feed, FeedError> {
        let Some(web_news) = self.db.get_web_news_by_name(name).await? else {
            return Err(FeedError::NoFeed);
        };

        if web_news.owner != user_id {
            return Err(FeedError::Access);
        }

        Ok(web_news.into())
    }

    pub async fn delete_feed(&self, user_id: i32, name: &str) -> Result<(), FeedError> {
        self.feed(user_id, name).await?;
        self.db.delete_web_news_by_name(name).await?;
        Ok(())
    }

    pub async fn update_feed(&self, user_id: i32, name: &str, feed: Feed) -> Result<(), FeedError> {
        self.feed(user_id, name).await?;

        self.db
            .update_web_news_by_name(
                name,
                &WebNews {
                    // ignored fields
                    id: -1,
                    name: "".into(),
                    owner: -1,

                    url: feed.url,
                    selector_post: feed.selectors.post,
                    selector_title: feed.selectors.title,
                    selector_link: feed.selectors.link,
                    selector_description: feed.selectors.description,
                    selector_date: feed.selectors.date,
                    selector_image: feed.selectors.image,
                },
            )
            .await?;

        Ok(())
    }
}
