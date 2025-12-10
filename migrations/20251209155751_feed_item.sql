CREATE TABLE IF NOT EXISTS feed_item (
    id SERIAL NOT NULL PRIMARY KEY,
    title VARCHAR NOT NULL,
    link VARCHAR NOT NULL,
    date TIMESTAMP WITH TIME ZONE NOT NULL,
    description VARCHAR,
    image_url VARCHAR,
    feed INT NOT NULL REFERENCES web_news (id)
)