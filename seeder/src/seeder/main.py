import os
from seeder.core import account
import psycopg2
from dotenv import load_dotenv, dotenv_values


def main():
    load_dotenv(".env")
    print(dotenv_values())
    conn = psycopg2.connect(
        dbname=os.getenv("DATABASE_NAME"),
        user=os.getenv("DATABASE_USER"),
        password=os.getenv("DATABASE_PASSWORD"),
        host=os.getenv("DATABASE_HOST"),
        port=os.getenv("DATABASE_PORT")
    )
    cur = conn.cursor()
    account.seed(cur)
    cur.close()
    conn.commit()
    conn.close()
