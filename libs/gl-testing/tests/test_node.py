from gltesting.identity import Identity
from gltesting.fixtures import *
from rich.pretty import pprint

def test_node_start(scheduler, clients, bitcoind):
    c = clients.new()
    res = c.register(configure=True)
    pprint(res)

    node_info = c.scheduler().schedule()
