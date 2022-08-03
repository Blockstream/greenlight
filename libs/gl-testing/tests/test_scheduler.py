from gltesting.fixtures import *
from rich.pretty import pprint

def test_scheduler_start(scheduler):
    pass


def test_scheduler_register(scheduler, clients):
    c = clients.new()
    r = c.register(configure=True)
    assert((c.directory / "device.crt").exists())
    assert((c.directory / "device-key.pem").exists())
    pprint(r)


def test_scheduler_recover(scheduler, clients):
    c = clients.new()
    r = c.register(configure=False)
    assert(not (c.directory / "device.crt").exists())
    assert(not (c.directory / "device-key.pem").exists())
    r = c.recover(configure=True)

    assert((c.directory / "device.crt").exists())
    assert((c.directory / "device-key.pem").exists())
