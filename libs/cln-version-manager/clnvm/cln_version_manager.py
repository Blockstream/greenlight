from dataclasses import dataclass
from io import BytesIO, StringIO
import hashlib
import logging
import os
from pathlib import Path
import subprocess
import sys
import tarfile
from typing import Dict, List, Optional, Tuple, Union
import shutil
from multiprocessing.pool import ThreadPool as Pool

import requests
from clnvm.utils import NodeVersion
from clnvm.errors import UnrunnableVersion, HashMismatch, VersionMismatch

PathLike = Union[os.PathLike, str]


@dataclass
class VersionDescriptor:
    tag: str
    url: str
    checksum: str


logger = logging.getLogger(__name__)

# Stores all versions of cln that can be scheduled
CLN_VERSIONS = [
    VersionDescriptor(
        tag="v0.10.1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v0.10.1.tar.bz2",
        checksum="928f09a46c707f68c8c5e1385f6a51e10f7b1e57c5cef52f5b73c7d661500af5",
    ),
    VersionDescriptor(
        tag="v0.10.2",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v0.10.2.tar.bz2",
        checksum="c323f2e41ffde962ac76b2aeaba3f2360b3aa6481028f11f12f114f408507bfe",
    ),
    VersionDescriptor(
        tag="v0.11.0.1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v0.11.0.1.tar.bz2",
        checksum="0f1a49bb8696db44a9ab93d8a226e82b4d3de03c9bae2eb38b750d75d4bcaceb",
    ),
    VersionDescriptor(
        tag="v0.11.2gl2",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v0.11.2gl2.tar.bz2",
        checksum="b15866b7beea239aaf4e38931fe09ee85bf2e58ad61c2ec79b83bb361364bf97",
    ),
    VersionDescriptor(
        tag="v0.11.2",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v0.11.2.tar.bz2",
        checksum="95209242d8ddc4879b959fb5e4594b4d2dcf7bac7227ec7c421ab05019de8633",
    ),
    VersionDescriptor(
        tag="v22.11gl1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v22.11gl1.tar.bz2",
        checksum="40b6c50babdc74d9fd251816efa46de0c12cac88d72e0c7b02457a8949d2690b",
    ),
    VersionDescriptor(
        tag="v23.05gl1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v23.05gl1.tar.bz2",
        checksum="e1a57a8ced59fd92703fad8e34926c014b71ee0c13cc7f863cb18b2ca19a58b9",
    ),
    VersionDescriptor(
        tag="v23.08gl1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v23.08gl1.tar.bz2",
        checksum="0e392c5117e14dc37cf72393f47657a09821f69ab8b45937d7e79ca8d91d17e9",
    ),
    VersionDescriptor(
        tag="v24.02gl1",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v24.02gl1.tar.bz2",
        checksum="7bbc7007231647df04ebb99b0316e5249b4372a8e0abe054e8fa4f7d97a001a8",
    ),
    VersionDescriptor(
        tag="v24.02",
        url="https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-v24.02.tar.bz2",
        checksum="690f5b3ce0404504913bb7cde22d88efeabe72226aefe31a70916cf89905455d",
    ),
]


_LIGHTNINGD_REL_PATH = Path("usr/local/bin/lightningd")
_BIN_REL_PATH = Path("usr/local/bin")


def _get_cln_version_path(cln_path: Optional[PathLike] = None) -> Path:
    """
    Retrieve the path where all cln-versions are stored
    """
    if cln_path is not None:
        return Path(cln_path).resolve()

    cln_cache_dir = os.environ.get("CLNVM_CACHE_DIR")
    if cln_cache_dir is not None:
        return Path(cln_cache_dir).resolve()

    xdg_cache_home = os.environ.get("XDG_CACHE_HOME")
    if xdg_cache_home is not None:
        return Path(xdg_cache_home).resolve() / "clnvm"

    else:
        return Path("~/.cache").expanduser().resolve() / "clnvm"


class ClnVersionManager:
    def __init__(
        self,
        cln_path: Optional[PathLike] = None,
        cln_versions: Optional[List[VersionDescriptor]] = None,
    ):
        self._cln_path: Path = _get_cln_version_path(cln_path)
        if cln_versions is not None:
            self._cln_versions = cln_versions
        else:
            self._cln_versions = CLN_VERSIONS

    def get_versions(self) -> List[VersionDescriptor]:
        """
        Retrieves the list of Core Lightning versions
        """
        return self._cln_versions

    def is_available(self, cln_version: VersionDescriptor) -> bool:
        target_path = self.get_target_path(cln_version)
        return os.path.exists(target_path)

    def get_all(self, force: bool = False) -> Dict[str, NodeVersion]:
        """
        Downloads all versions of Core Lightning

        Args:
            force (False): If force is True the data will be overriden
        """
        versions = list(self.get_versions())

        def do_download(
            version: VersionDescriptor,
        ) -> Optional[Tuple[str, NodeVersion]]:
            try:
                return version.tag, self.get(version, force)
            except Exception:
                logger.exception(
                    "Failed to download %s. This version will be ignored", version.tag
                )
                return None

        def do() -> List[Tuple[str, NodeVersion]]:
            with Pool(10) as p:
                data = p.map(do_download, versions)

            return [t for t in data if t is not None]

        return dict(do())

    def get_target_path(self, cln_version: VersionDescriptor) -> Path:
        """Computes the path corresponding to which a cln version should be downloaded"""
        return Path(self._cln_path) / cln_version.checksum / cln_version.tag

    def get_descriptor_from_tag(self, tag: str) -> VersionDescriptor:
        cln_dict = {d.tag: d for d in CLN_VERSIONS}
        descriptor = cln_dict.get(tag, None)

        if descriptor is None:
            raise ValueError(f"Failed to find version {tag}")

        return descriptor

    def latest(self) -> NodeVersion:
        vs = [d.tag for d in self.get_versions()]
        latest = max(vs)
        descriptor = self.get_descriptor_from_tag(latest)
        return self.get(descriptor)

    def get(self, cln_version: VersionDescriptor, force: bool = False) -> NodeVersion:
        """
        Ensures the provided version exists.
        It returns the path to the corresponding binary
        """
        target_path = self.get_target_path(cln_version)

        if not os.path.exists(target_path):
            self._download(cln_version, target_path)
        elif force:
            shutil.rmtree(target_path)
            self._download(cln_version, target_path)
        else:
            # Files are already downloaded
            # We don't do anything
            pass

        return NodeVersion(
            name=cln_version.tag,
            lightningd=target_path / _LIGHTNINGD_REL_PATH,
            bin_path=target_path / _BIN_REL_PATH,
            root_path=target_path,
        )

    def _download(self, cln_version: VersionDescriptor, target_path: Path, verify_tag:bool = False) -> None:
        """Downloads the provided cln_version"""
        tag = cln_version.tag
        logger.info("Downloading version %s to %s", tag, target_path)

        # Retrieve the version from the provided url
        response = requests.get(cln_version.url)
        if response.status_code != 200:
            logger.warning(
                "Failed to retrieve %s: %s - %s",
                tag,
                response.status_code,
                response.content[:124],
            )
            raise Exception(f"Failed to find version {tag}")

        data = response.content

        # We check the hash of the downloaded data
        # If the hash doesn't match we stop and alert the user
        m = hashlib.sha256()
        m.update(data)
        content_sha = m.hexdigest()

        logger.debug("Downloaded version %s with checksum %s", tag, content_sha)
        ignore_hash = bool(os.environ.get("GL_TESTING_IGNORE_HASH", False))
        if ignore_hash:
            logger.warning(
                "Checking the hash of remote versions is disabled which is unsafe. "
                "Try to unset GL_TESTING_IGNORE_HASH"
            )

        if (not ignore_hash) and content_sha != cln_version.checksum:
            raise HashMismatch(
                tag=cln_version.tag, expected=cln_version.checksum, actual=content_sha
            )

        # We extract the downloaded tar-file in the section below.
        # Note, that we never put the tar-file on disk. We extract
        # it straight from memory
        #
        # In python 3.12 the `filter`-argument was introduced
        # to `TarFile.extractall`.
        #
        # Using this argument provide us extra security against
        # malicious .tar-files.
        #
        # The extra security is nice to have. We used the hash to
        # check the authenticity of our .tar-files a few lines above.
        #
        # We'll use the filter argument if it is available.
        # In the other case we rely on the hash to keep our users safe
        content_fh = BytesIO(data)
        tf = tarfile.open(fileobj=content_fh)

        current_version = sys.version_info
        if current_version.minor >= 12:
            tf.extractall(path=target_path, filter="data")
        else:
            tf.extractall(path=target_path)

        if verify_tag:
            try:
                # We verify if the path matches the version
                lightningd_path = target_path / _LIGHTNINGD_REL_PATH
                version = (
                    subprocess.check_output([lightningd_path, "--version"])
                    .strip()
                    .decode("ASCII")
                )
            except Exception as e:
                # Clean-up the bad version
                shutil.rmtree(target_path)
                raise UnrunnableVersion(tag=cln_version.tag) from e

            if version != tag:
                # Clean-up the bad version
                shutil.rmtree(target_path)
                raise VersionMismatch(expected=tag, actual=version)

        return
