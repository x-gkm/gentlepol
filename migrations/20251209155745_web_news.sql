CREATE TABLE IF NOT EXISTS web_news (
    id SERIAL NOT NULL PRIMARY KEY,
    url VARCHAR NOT NULL,
    name VARCHAR NOT NULL UNIQUE,
    owner INT NOT NULL REFERENCES feed_user (id),
    selector_post VARCHAR,
    selector_title VARCHAR,
    selector_link VARCHAR NOT NULL,
    selector_description VARCHAR,
    selector_date VARCHAR,
    selector_image VARCHAR
)