from gltesting.fixtures import *

def test_setup(clients, node_factory, scheduler, directory, bitcoind):
    """Sets up a gltesting backend and a small lightning network.

    This is meant to be run from inside the docker shell. See the 
    gltesting tutorial for further info.
    """
    l1, l2, l3 = node_factory.line_graph(3)  # (1)!
	
    # Assuming we want interact with l3 we'll want to print
	# its contact details:
    print(f"l3 details: {l3.info['id']} @ 127.0.0.1:{l3.daemon.port}")
    print()
    print(f"export GL_CA_CRT={directory}/certs/ca.pem")  # (4)
    print(f"export GL_NOBODY_CRT={directory}/certs/users/nobody.crt")
    print(f"export GL_NOBODY_KEY={directory}/certs/users/nobody-key.pem")
    print(f"export GL_SCHEDULER_GRPC_URI=https://localhost:{scheduler.grpc_port}")  # (3)!
	
    breakpoint()  # (2)!