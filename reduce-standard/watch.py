import threading
from time import sleep
import subprocess
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

COMMAND_UPDATE_PYREDUCE = "pdm update pyreduce"
COMMAND_RUN_SERVER = "pdm run python -m reduce_standard"

DEPENDENT_PROJECTS = (
    "../reduce-core",
    "../pyreduce",
)

DEBOUNCE_TIME = 1.0

class MyHandler(FileSystemEventHandler):
    def __init__(self, server_process):
        self.server_process = server_process
        self.timer = None
        self.should_update_pyreduce = False

    def run_server(self):
        if self.should_update_pyreduce:
            update_process =subprocess.Popen(COMMAND_UPDATE_PYREDUCE, shell=True)
            update_process.wait()
            self.should_update_pyreduce = False

        self.server_process.terminate()
        self.server_process = subprocess.Popen(COMMAND_RUN_SERVER, shell=True)

    def on_any_event(self, event):
        if event.event_type not in ['modified', 'created', 'deleted']:
            return
        if event.src_path.startswith("../"):
            if self.timer is not None and self.timer.is_alive():
                self.timer.cancel()
            self.should_update_pyreduce = True
            self.timer = threading.Timer(DEBOUNCE_TIME, self.run_server)
            self.timer.start()
            return

        if event.src_path.startswith("./"):
            if self.timer is not None and self.timer.is_alive():
                self.timer.cancel()
            self.timer = threading.Timer(DEBOUNCE_TIME, self.run_server)
            self.timer.start()

def main():
    update_process =subprocess.Popen(COMMAND_UPDATE_PYREDUCE, shell=True)
    update_process.wait()
    event_handler = MyHandler(subprocess.Popen(COMMAND_RUN_SERVER, shell=True))
    observer = Observer()
    observer.schedule(event_handler, '.', recursive=True)
    for project in DEPENDENT_PROJECTS:
        observer.schedule(event_handler, project, recursive=True)

    observer.start()
    try:
        while True:
            sleep(1)
    finally:
        observer.stop()
        observer.join()

if __name__ == "__main__":
    main()
