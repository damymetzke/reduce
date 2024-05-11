from pyreduce import start_server
from time import sleep

if __name__ != "__main__":
    exit(0)

handle = start_server()

while True:
    sleep(5)
