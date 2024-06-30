# Reduce: Improve productivity by reducing complexity
# Copyright (C) 2024  Damy Metzke
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

from psycopg2.extensions import cursor
from faker import Faker
from argon2 import PasswordHasher

def seed(cur: cursor):
    fake = Faker()
    ph = PasswordHasher()

    cur.execute("""
    WITH inserted AS (INSERT INTO accounts DEFAULT VALUES RETURNING id)
    INSERT INTO email_password_logins (account_id, email, password_hash)
    SELECT id, %s, %s from inserted;
    """,
                ("user@example.com", ph.hash("password")))

    for _ in range(10):
        cur.execute("""
        WITH inserted AS (INSERT INTO accounts DEFAULT VALUES RETURNING id)
        INSERT INTO email_password_logins (account_id, email, password_hash)
        SELECT id, %s, %s from inserted;
        """,
                    (fake.email(), ph.hash("password")))
