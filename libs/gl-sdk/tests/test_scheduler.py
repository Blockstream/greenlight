"""Testing client-scheduler interactions"""

import pytest
import glsdk
from gltesting.fixtures import *


def test_register(scheduler, clients, tmp_path):
    signer = glsdk.Signer(" ".join(["abandon"] * 11 + ["about"]), str(tmp_path / "device-state"))
    signer.start()

    # Check that the derived node_id matches the Signer seed phrase.
    assert (
        signer.node_id().hex()
        == "03653e90c1ce4660fd8505dd6d643356e93cfe202af109d382787639dd5890e87d"
    )
