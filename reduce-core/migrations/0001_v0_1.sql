CREATE TABLE accounts(
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(100) NOT NULL
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    account_id INT NOT NULL REFERENCES accounts(id),
    session_token VARCHAR(44) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    csrf_token_1 VARCHAR(22) NOT NULL,
    csrf_token_2 VARCHAR(22) NOT NULL,
    csrf_token_1_expiration TIMESTAMP NOT NULL,
    csrf_token_2_expiration TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    csrf_token_refresh TIMESTAMP NOT NULL
);

CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(6) NOT NULL
);

CREATE TABLE time_entries (
    project_id INT REFERENCES projects(id) NOT NULL,
    day DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME,
    PRIMARY KEY(day, start_time)
);

CREATE TABLE time_comments (
    project_id INT REFERENCES projects(id) NOT NULL,
    day DATE NOT NULL,
    content TEXT,
    PRIMARY KEY(project_id, day)
);

CREATE TABLE upkeep_items (
  id SERIAL PRIMARY KEY,
  description VARCHAR(255) NOT NULL,
  cooldown_days INT NOT NULL,
  due DATE NOT NULL
);
