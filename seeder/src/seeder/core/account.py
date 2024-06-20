from psycopg2.extensions import cursor
from faker import Faker
from argon2 import PasswordHasher

def seed(cur: cursor):
    fake = Faker()
    ph = PasswordHasher()

    cur.execute("""
    INSERT INTO accounts (email, password_hash)
    VALUES (%s, %s)
    """,
                ("user@example.com", ph.hash("password")))

    for _ in range(10):
        cur.execute("""
        INSERT INTO accounts (email, password_hash)
        VALUES (%s, %s)
        """,
                    (fake.email(), ph.hash("password")))
