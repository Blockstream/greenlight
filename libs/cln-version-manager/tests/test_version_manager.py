from unittest import mock
import pytest
import shutil
import os

import requests

from clnvm.cln_version_manager import (
    ClnVersionManager,
    VersionDescriptor,
    version_base,
    version_sort_key,
)


def _descriptor(tag: str) -> VersionDescriptor:
    return VersionDescriptor(tag=tag, url="", checksum=tag)


# Mirrors the kind of tags found in the production manifest, deliberately out
# of order.
_MANIFEST_TAGS = [
    "main",
    "v0.11.2gl2",
    "v23.08.",
    "v23.08gl1",
    "v24.11gl1",
    "v25.12.",
    "v25.12gl1",
    "v26.06gl1",
]


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


def test_version_sort_key() -> None:
    # ``main`` and other non-numbered tags are not orderable.
    assert version_sort_key("main") is None

    # The glN suffix is captured separately, and absent suffixes sort below
    # their greenlight counterpart of the same base.
    assert version_sort_key("v25.12.") == ((25, 12), -1)
    assert version_sort_key("v25.12gl1") == ((25, 12), 1)
    assert version_sort_key("v0.11.2gl2") == ((0, 11, 2), 2)
    assert version_sort_key("v25.12.") < version_sort_key("v25.12gl1")
    assert version_sort_key("v25.12gl1") < version_sort_key("v26.06gl1")


def test_version_base_ignores_suffix() -> None:
    # Base comparison ignores the glN suffix, so the signer reporting
    # ``v25.12`` matches both the plain and greenlight builds.
    assert version_base("v25.12") == version_base("v25.12gl1") == (25, 12)
    assert version_base("v25.12.") == (25, 12)
    assert version_base("main") is None


def test_supported_versions_filters_and_sorts() -> None:
    vm = ClnVersionManager(
        cln_versions=[_descriptor(t) for t in _MANIFEST_TAGS]
    )

    supported = vm.supported_versions("v23.08", "v25.12")
    tags = [d.tag for d in supported]

    # ``main`` dropped (not numbered), ``v0.11.2gl2`` below the lower bound,
    # ``v26.06gl1`` above the signer-supported upper bound. Result is sorted
    # ascending with the greenlight build after the plain build of the same
    # base.
    assert tags == [
        "v23.08.",
        "v23.08gl1",
        "v24.11gl1",
        "v25.12.",
        "v25.12gl1",
    ]


def test_supported_versions_latest_is_greenlight_build() -> None:
    vm = ClnVersionManager(
        cln_versions=[_descriptor(t) for t in _MANIFEST_TAGS]
    )

    # The newest supported version is the greenlight build at the upper
    # bound, never the newer-but-unsupported v26.06gl1.
    assert vm.supported_versions("v23.08", "v25.12")[-1].tag == "v25.12gl1"
