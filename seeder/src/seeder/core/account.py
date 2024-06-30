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
