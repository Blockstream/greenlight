from gltesting.fixtures import *
from rich.pretty import pprint

def test_scheduler_start(scheduler):
    pass


def test_scheduler_register(scheduler, clients):
    c = clients.new()
    signer = c.signer()
    # Use the signer to sign registration requests.
    r = c.scheduler().register(signer)
    pprint(r)
