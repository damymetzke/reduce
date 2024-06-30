CREATE TABLE accounts(
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE bootstrap_keys(
    key VARCHAR(44) NOT NULL PRIMARY KEY,
    account_id INT NOT NULL REFERENCES accounts(id),
    consumed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE email_password_logins(
    account_id INT NOT NULL PRIMARY KEY REFERENCES accounts(id),
    email VARCHAR(255) NOT NULL UNIQUE,
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
  account_id INT NOT NULL REFERENCES accounts(id),
  description VARCHAR(255) NOT NULL,
  cooldown_days INT NOT NULL,
  due DATE NOT NULL
);
