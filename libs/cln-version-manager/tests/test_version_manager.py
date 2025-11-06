from unittest import mock
import pytest
import shutil
import os

import requests

from clnvm.cln_version_manager import ClnVersionManager, VersionDescriptor


def get_tmp_dir(name: str) -> str:
    tmp_dir = f"/tmp/gl-testing/case/{name}"
    if os.path.exists(tmp_dir):
        shutil.rmtree(tmp_dir)
    os.makedirs(tmp_dir)
    return tmp_dir


def test_download_cln_version() -> None:
    # Download the latest 2 versions from the manifest
    vm = ClnVersionManager(cln_path=get_tmp_dir("test_download_cln_version"))
    # Get all versions from the manifest and use the last 2
    all_versions = vm.get_versions()
    versions = all_versions[-2:] if len(all_versions) >= 2 else all_versions

    vm_test = ClnVersionManager(
        cln_versions=versions, cln_path=get_tmp_dir("test_download_cln_version")
    )
    vm_test.get_all()

    # get them again to verify we don't download them
    with mock.patch("requests.get") as request_mock:
        vm_test.get_all()
        assert not request_mock.get.called
