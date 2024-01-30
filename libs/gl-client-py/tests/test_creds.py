import pytest
from fixtures import *
from glclient import Scheduler, Credentials, TlsConfig

CRED_RUNE = bytes(
    '"\xec\x01q3gAOP3JVkuen3qOh2G3LMU9TbHpXQS2VXfecJmlZY89MC1nbDAmcHVia2V5PTA0OGI0ZWZhNDZkNTZmMmUxM2RmOTdjOGFmNzJiZjYzZWEwNDgzODFlMTdkMTRhOGVkMThlNDVhMzFkNDIzMmNlMzE3OWE4NjE2ZTU2ODUxOTc5MjcxOTZlMTI2YjU0YjhhMmU5NzAwNWJiNzY2YTYzM2M1ODc0M2RjMGU3ZDZhZGY=',
    "utf-8",
)


def test_upgrade_credentials(scheduler, sclient, signer):
    creds = sclient.register(signer).creds
    screds = Credentials.as_device().from_bytes(creds).build()

    # Remove rune from creds.
    creds = creds[0 : (len(creds) - len(CRED_RUNE)) + 1]

    with pytest.raises(ValueError):
        Credentials.as_device().from_bytes(creds).build()

    c = (
        Credentials.as_device()
        .from_bytes(creds)
        .upgrade(
            Scheduler(
                node_id=signer.node_id(), network="regtest", tls=TlsConfig(screds)
            ).inner,
            signer.inner,
        )
        .build()
    )

    assert c
    assert type(c.to_bytes()) is bytes
