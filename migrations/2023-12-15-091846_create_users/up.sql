CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    creation_date TIMESTAMP NOT NULL,
    last_login_date TIMESTAMP NOT NULL
);
