# We do not import `gl-testing` or `pyln-testing` since the
# `gl-testserver` is intended to run tests externally from a python
# environment. We will use `gl-client-py` to interact with it though.
# Ok, one exception, `TailableProc` is used to run and tail the
# `gl-testserver`.

from pathlib import Path
from pyln.testing.utils import TailableProc
import json
import logging
import os
import pytest
import shutil
import signal
import tempfile
import time


@pytest.fixture
def test_name(request):
    yield request.function.__name__


@pytest.fixture(scope="session")
def test_base_dir():
    d = os.getenv("TEST_DIR", "/tmp")
    directory = tempfile.mkdtemp(prefix="ltests-", dir=d)
    print("Running tests in {}".format(directory))

    yield directory


@pytest.fixture
def directory(request, test_base_dir, test_name):
    """Return a per-test specific directory.

    This makes a unique test-directory even if a test is rerun multiple times.

    """
    directory = os.path.join(test_base_dir, test_name)
    request.node.has_errors = False

    if not os.path.exists(directory):
        os.makedirs(directory)

    yield directory

    # This uses the status set in conftest.pytest_runtest_makereport to
    # determine whether we succeeded or failed. Outcome can be None if the
    # failure occurs during the setup phase, hence the use to getattr instead
    # of accessing it directly.
    rep_call = getattr(request.node, "rep_call", None)
    outcome = "passed" if rep_call is None else rep_call.outcome
    failed = not outcome or request.node.has_errors or outcome != "passed"

    if not failed:
        try:
            shutil.rmtree(directory)
        except OSError:
            # Usually, this means that e.g. valgrind is still running.  Wait
            # a little and retry.
            files = [
                os.path.join(dp, f) for dp, dn, fn in os.walk(directory) for f in fn
            ]
            print("Directory still contains files: ", files)
            print("... sleeping then retrying")
            time.sleep(10)
            shutil.rmtree(directory)
    else:
        logging.debug(
            "Test execution failed, leaving the test directory {} intact.".format(
                directory
            )
        )


class TestServer(TailableProc):
    def __init__(self, directory):
        TailableProc.__init__(self, outputDir=directory)
        self.cmd_line = [
            "python3",
            str(Path(__file__).parent / ".." / "gltestserver" / "__main__.py"),
            "run",
            f"--directory={directory}",
        ]
        self.directory = Path(directory)

    def start(self):
        TailableProc.start(self)
        self.wait_for_log(r"Ctrl-C")

    def stop(self):
        self.proc.send_signal(signal.SIGTERM)
        self.proc.wait()

    def metadata(self):
        metadata = json.load(
            (self.directory / "gl-testserver" / "metadata.json").open(mode="r")
        )
        return metadata


@pytest.fixture
def testserver(directory):
    ts = TestServer(directory=directory)
    ts.start()

    yield ts
    ts.stop()


def test_start(testserver):
    print(testserver.metadata())
