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
    csrf_token VARCHAR(24) NOT NULL
);

CREATE TABLE upkeep_items (
  id SERIAL PRIMARY KEY,
  description VARCHAR(255) NOT NULL,
  cooldown_days INT NOT NULL,
  due DATE NOT NULL
);
