/*
* Reduce: Improve productivity by reducing complexity
* Copyright (C) 2024  Damy Metzke
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
