use serde::{Deserialize, Serialize};

use crate::db::WebNews;

pub mod db;

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
