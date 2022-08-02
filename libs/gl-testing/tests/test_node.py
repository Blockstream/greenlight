from gltesting.fixtures import *
from rich.pretty import pprint

def test_node_start(scheduler, clients, bitcoind):
    c = clients.new()
    cs = c.signer()
    res = c.scheduler().register(cs)
    pprint(res)
