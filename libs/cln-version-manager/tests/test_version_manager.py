from unittest import mock
import pytest
import shutil
import os

import requests

from clnvm.cln_version_manager import ClnVersionManager
from clnvm.cln_version_manager import CLN_VERSIONS


def get_tmp_dir(name: str) -> str:
    tmp_dir = f"/tmp/gl-testing/case/{name}"
    if os.path.exists(tmp_dir):
        shutil.rmtree(tmp_dir)
    os.makedirs(tmp_dir)
    return tmp_dir


def test_download_cln_version() -> None:
    # Only download the first 2 versions in the test
    versions = CLN_VERSIONS[-2:]
    vm = ClnVersionManager(
        cln_versions=versions, cln_path=get_tmp_dir("test_download_cln_version")
    )
    vm.get_all()

    # get them again to verify we don't download them
    with mock.patch("requests.get") as request_mock:
        vm.get_all()
        assert not request_mock.get.called

    # get them again using force to ensure we do download them
    vm.get_all(force=True)
