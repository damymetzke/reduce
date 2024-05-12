from pyreduce import ServerConfig
from time import sleep

if __name__ != "__main__":
    exit(0)

config = ServerConfig()
config.set_database_url("postgres://user:password@localhost:5432/reduce_dev")
config.set_server_bind_address("0.0.0.0:3000")

handle = config.start_server()

while True:
    sleep(5)
