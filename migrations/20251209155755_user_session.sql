CREATE TABLE IF NOT EXISTS user_session (
    owner INT NOT NULL REFERENCES feed_user (id),
    token UUID NOT NULL,
    valid_until TIMESTAMP WITH TIME ZONE NOT NULL
)