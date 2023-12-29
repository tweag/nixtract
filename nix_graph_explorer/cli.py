import subprocess
import requests
import backoff
import shutil
from gremlin_python.process.anonymous_traversal import traversal
from gremlin_python.driver.driver_remote_connection import DriverRemoteConnection
from requests.exceptions import ConnectionError
from nixtract.model import Derivation
import subprocess

import psutil


# TODO: Refactor this to make it a bit more re-usable (will be shared with front-end)
class GraphProcess:
    def __init__(self, cmd: str = "janusgraph-server", host="0.0.0.0", port=8182):
        self.cmd = cmd
        self.host = host
        self.port = port
        self._proc: subprocess.Popen | None = None

    def cmd_exists(self) -> bool:
        if shutil.which(self.cmd) is not None:
            return True
        else:
            return False

    def open(self) -> None:
        if not self.cmd_exists():
            raise Exception(
                f"The specified command {self.cmd} could not be found in the current environment. "
                "Make sure that you have it installed and on the PATH."
            )
        if self._proc:
            return
        self._proc = subprocess.Popen(
            self.cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )

    def close(self) -> None:
        if self._proc:
            for child in psutil.Process(self._proc.pid).children(recursive=True):
                child.kill()
            self._proc.kill()
            self._proc.wait(timeout=30)

    def __enter__(self):
        self.open()
        self.wait_until_ready()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
        return False

    @backoff.on_predicate(backoff.expo, lambda is_ready: not is_ready, max_tries=10)
    def wait_until_ready(self) -> bool:
        return self.is_ready()

    def is_ready(self) -> bool:
        try:
            requests.get(f"http://{self.host}:{self.port}")
            return True
        except ConnectionError:
            return False


# TODO: Make this a proper CLI...
def cli():
    with GraphProcess() as graph:
        if not graph.cmd_exists():
            raise Exception("Could not find the command!")
        g = traversal().with_remote(
            DriverRemoteConnection(f"ws://{graph.host}:{graph.port}/gremlin", "g")
        )

        print(g.V().count().to_list())

        # TODO: Ingest some data!

        # Aggressively read in the input file, we should probably do some batching...
        with open("./example-graph-for-jedi.jsonl") as f:
            lines = f.readlines()
        objs = []
        for l in lines:
            try:
                objs.append(Derivation.parse_raw(l))
            except:
                continue
        # for obj in objs:
        # Recurse object and generate graph entities
        print(objs[0])
