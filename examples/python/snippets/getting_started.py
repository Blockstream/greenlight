import os
import sys
from pathlib import Path
import bip39  # type: ignore
from glclient import Credentials, Signer, Scheduler  # type: ignore
from pathlib import Path
from pyln import grpc as clnpb  # type: ignore
import secrets  # Make sure to use cryptographically sound randomness


# ---8<--- [start: upgrade_device_certs_to_creds]
def upgrade_device_certs_to_creds(
    scheduler: Scheduler, signer: Signer, device_cert: bytes, device_key: bytes
):
    device_creds = Credentials.from_parts(device_cert, device_key, "")
    return device_creds.upgrade(scheduler.inner, signer.inner)
# ---8<--- [end: upgrade_device_certs_to_creds]


def save_to_file(file_name: str, data: bytes) -> None:
    with open(file_name, "wb") as file:
        file.write(data)


def read_file(file_name: str) -> bytes:
    with open(file_name, "rb") as file:
        return file.read()


def create_seed() -> bytes:
    # ---8<--- [start: create_seed]
    rand = secrets.randbits(256).to_bytes(32, "big")  # 32 bytes of randomness

    # Show seed phrase to user
    phrase = bip39.encode_bytes(rand)

    seed = bip39.phrase_to_seed(phrase)[:32]  # Only need the first 32 bytes

    # Store the seed on the filesystem, or secure configuration system
    save_to_file("seed", seed)

    # ---8<--- [end: create_seed]
    return seed

def register_node(seed: bytes, developer_cert_path: str, developer_key_path: str) -> None:
    # ---8<--- [start: dev_creds]
    developer_cert = Path(developer_cert_path).open(mode="rb").read()
    developer_key = Path(developer_key_path).open(mode="rb").read()

    developer_creds = Credentials.nobody_with(developer_cert, developer_key)
    # ---8<--- [end: dev_creds]

    # ---8<--- [start: init_signer]
    network = "regtest"
    signer = Signer(seed, network, developer_creds)
    # ---8<--- [end: init_signer]

    # ---8<--- [start: register_node]
    scheduler = Scheduler(network, developer_creds)

    # Passing in the signer is required because the client needs to prove
    # ownership of the `node_id`
    registration_response = scheduler.register(signer, invite_code=None)

    # ---8<--- [start: device_creds]
    device_creds = Credentials.from_bytes(registration_response.creds)
    save_to_file("creds", device_creds.to_bytes())
    # ---8<--- [end: device_creds]

    # ---8<--- [end: register_node]

    return scheduler, device_creds, signer


def get_node(scheduler: Scheduler, device_creds: Credentials) -> dict:
    # ---8<--- [start: get_node]
    scheduler = scheduler.authenticate(device_creds)
    node = scheduler.node()
    # ---8<--- [end: get_node]
    return node


# TODO: Remove node_id from signature and add node_id to credentials
def start_node(device_creds_path: str, node_id: bytes) -> None:
    # ---8<--- [start: start_node]
    network = "regtest"
    device_creds = Credentials.from_path(device_creds_path)
    scheduler = Scheduler(network, device_creds)

    node = scheduler.node()
    # ---8<--- [end: start_node]

    # ---8<--- [start: list_peers]
    info = node.get_info()
    peers = node.list_peers()
    # ---8<--- [end: list_peers]

    # ---8<--- [start: start_signer]
    seed = read_file("seed")
    signer = Signer(seed, network, device_creds)

    signer.run_in_thread()
    # ---8<--- [end: start_signer]

    # ---8<--- [start: create_invoice]
    node.invoice(
        amount_msat=clnpb.AmountOrAny(amount=clnpb.Amount(msat=10000)),
        description="description",
        label="label",
    )
    # ---8<--- [end: create_invoice]


def recover_node(developer_cert: bytes, developer_key: bytes) -> None:
    # ---8<--- [start: recover_node]
    seed = read_file("seed")
    network = "regtest"
    signer_creds = Credentials.nobody_with(developer_cert, developer_key)
    signer = Signer(seed, network, signer_creds)

    scheduler = Scheduler(
        network,
        signer_creds,
    )

    scheduler_creds = signer_creds.upgrade(scheduler.inner, signer.inner)

    scheduler = Scheduler(
        network,
        scheduler_creds,
    )

    scheduler.recover(signer)
    # ---8<--- [end: recover_node]

def main():
    NETWORK = "regtest"
    base_dir = "/tmp/gltests/gl-testserver/certs/users"
    developer_cert_path = Path(base_dir) / "nobody.crt"
    developer_key_path = Path(base_dir) / "nobody-key.pem"

    # Verify files exist
    if not os.path.exists(developer_cert_path):
        print(f"Error: Developer certificate not found at {developer_cert_path}")
        sys.exit(1)
        
    if not os.path.exists(developer_key_path):
        print(f"Error: Developer key not found at {developer_key_path}")
        sys.exit(1)
    
    # Step 1: Create seed
    print("Creating seed...")
    seed = create_seed()
    
    # Step 2: Register node
    print("Registering node...")
    my_scheduler, device_creds, signer = register_node(seed, developer_cert_path, developer_key_path)
    
    # Step 3: Get GL node
    print("Getting GL node info...")
    node = get_node(my_scheduler, device_creds)

    # Step 3: Get lightning node's information
    print("Fetching lightning node getinfo...")
    info = node.get_info()
    print('Node information:')
    print(info)
    
    # Remove the assert False to let the script complete normally
    print("Script completed successfully!")

if __name__ == "__main__":
    main()