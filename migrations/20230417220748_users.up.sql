CREATE TABLE IF NOT EXISTS users (
    username       TEXT  PRIMARY KEY  NOT NULL,
    name           TEXT               NOT NULL,
    password_hash  TEXT               NOT NULL
);

INSERT INTO users (username, name, password_hash) VALUES ('admin', 'Administrator', '$2b$12$4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4O8Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q4Q');
