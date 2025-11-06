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
from clnvm.errors import (
    UnrunnableVersion,
    HashMismatch,
    VersionMismatch,
    SignatureVerificationFailed,
)

PathLike = Union[os.PathLike, str]

# Unless the URL is an absolute URL, use this prefix to complete the URL.
BASE_URL = "https://storage.googleapis.com/greenlight-artifacts/cln"
MANIFEST_URL = f"{BASE_URL}/manifest.json"
PUBKEY_FINGERPRINT = "1E832A80A25B69C6C7123285AB5187B13F8DD139"
PUBKEY_URL = f"{BASE_URL}/{PUBKEY_FINGERPRINT}.pub"
_LIGHTNINGD_REL_PATH = Path("usr/local/bin/lightningd")
_BIN_REL_PATH = Path("usr/local/bin")
GPG_OPTS = ['--no-default-keyring', '--keyring=/tmp/clnvm-keyring.gpg']

@dataclass
class VersionDescriptor:
    tag: str
    url: str
    checksum: str
    signature: Optional[str] = None


logger = logging.getLogger(__name__)


def _get_cache_dir() -> Path:
    cln_cache_dir = os.environ.get("CLNVM_CACHE_DIR")
    if cln_cache_dir is not None:
        return Path(cln_cache_dir).resolve()

    xdg_cache_home = os.environ.get("XDG_CACHE_HOME")
    if xdg_cache_home is not None:
        return Path(xdg_cache_home).resolve() / "clnvm"

    else:
        return Path("~/.cache").expanduser().resolve() / "clnvm"

def _ensure_pubkey() -> Path:
    """Ensure we have the pubkey that signs the releases in the cache.

    Download if we don't yet.
    """
    fpath = _get_cache_dir()
    fpath.mkdir(parents=True, exist_ok=True)
    pubkey_path = fpath / f"{PUBKEY_FINGERPRINT}.asc"
    if not pubkey_path.exists():
        logger.debug("Fetching public key from %s", PUBKEY_URL)
        pubkey_response = requests.get(PUBKEY_URL)
        if pubkey_response.status_code != 200:
            raise ValueError(
                f"Failed to fetch public key: {pubkey_response.status_code}"
            )
        with Path(pubkey_path).open(mode="w") as f:
            f.write(pubkey_response.text)
        try:
            subprocess.check_call(["gpg", *GPG_OPTS, "--import", pubkey_path])
        except:
            raise ValueError("Failed to import public key")
        logger.debug("Imported public key.")

    return pubkey_path

def _verify_signature(
    file_path: Path,
    signature: str,
    tag: str,
    pubkey_fingerprint: str = PUBKEY_FINGERPRINT,
) -> None:
    """
    Verify the GPG signature of a file.

    Args:
        file_path: Path to the file to verify
        signature: The ASCII-armored PGP signature
        tag: Version tag (for error messages)
        pubkey_fingerprint: The fingerprint of the public key to use

    Raises:
        SignatureVerificationFailed: If signature verification fails
    """
    logger.debug("Verifying GPG signature for version %s", tag)

    pubkey_path = _ensure_pubkey()
    sig_file = Path(file_path.parent / "signature.asc")
    with sig_file.open(mode="w") as f:
        f.write(signature)
    try:
        subprocess.check_call(["gpg", *GPG_OPTS, "--verify", sig_file, file_path])
    except Exception as e:
        raise SignatureVerificationFailed(
            tag=tag, reason=f"Invalid signature: {repr(e)}"
        )

    logger.debug(
        "Successfully verified signature for version %s with key",
        tag,
    )


def _get_cln_version_path(cln_path: Optional[PathLike] = None) -> Path:
    """
    Retrieve the path where all cln-versions are stored
    """
    if cln_path is not None:
        return Path(cln_path).resolve()
    return _get_cache_dir()


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
            self.update()

    def update(self) -> None:
        """Fetch the manifest, and populate our list of versions."""
        manifest = requests.get(MANIFEST_URL).json()
        versions = [
            VersionDescriptor(
                tag=k,
                url=f"{BASE_URL}/{v['filename']}",
                checksum=v["sha256"],
                signature=v.get("signature"),
            )
            for k, v in manifest["versions"].items()
        ]
        self._cln_versions = versions

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
        cln_dict = {d.tag: d for d in self._cln_versions}
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

    def _download(
        self,
        cln_version: VersionDescriptor,
        target_path: Path,
        verify_tag: bool = False,
    ) -> None:
        """Downloads the provided cln_version"""
        tag = cln_version.tag
        logger.info("Downloading version %s to %s", tag, target_path)

        # Retrieve the version from the provided url
        response = requests.get(cln_version.url, stream=True)
        if response.status_code != 200:
            logger.warning(
                "Failed to retrieve %s: %s - %s",
                tag,
                response.status_code,
                response.content[:124],
            )
            raise Exception(f"Failed to find version {tag}")

        # Write to a temporary file and compute hash in one pass
        import tempfile

        tmp_dir = tempfile.mkdtemp()
        tmp_file = Path(tmp_dir) / "download.tar"

        try:
            m = hashlib.sha256()
            with open(tmp_file, "wb") as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
                        m.update(chunk)

            content_sha = m.hexdigest()
            logger.debug("Downloaded version %s with checksum %s", tag, content_sha)

            # Verify signature if available
            if cln_version.signature:
                logger.info("Verifying GPG signature for version %s", tag)
                _verify_signature(tmp_file, cln_version.signature, tag)
            else:
                logger.warning("No signature available for version %s", tag)

            # Check the hash
            ignore_hash = bool(os.environ.get("GL_TESTING_IGNORE_HASH", False))
            if ignore_hash:
                logger.warning(
                    "Checking the hash of remote versions is disabled which is unsafe. "
                    "Try to unset GL_TESTING_IGNORE_HASH"
                )

            if (not ignore_hash) and content_sha != cln_version.checksum:
                raise HashMismatch(
                    tag=cln_version.tag,
                    expected=cln_version.checksum,
                    actual=content_sha,
                )

            # We extract the downloaded tar-file in the section below.
            # In python 3.12 the `filter`-argument was introduced
            # to `TarFile.extractall`.
            #
            # Using this argument provide us extra security against
            # malicious .tar-files.
            #
            # The extra security is nice to have. We used the hash and
            # signature to check the authenticity of our .tar-files above.
            #
            # We'll use the filter argument if it is available.
            # In the other case we rely on the hash to keep our users safe
            tf = tarfile.open(str(tmp_file))

            current_version = sys.version_info
            if current_version.minor >= 12:
                tf.extractall(path=target_path, filter="data")
            else:
                tf.extractall(path=target_path)

        finally:
            # Clean up temporary file
            shutil.rmtree(tmp_dir)

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
