import os
import sys
import string
from pathlib import Path
import bip39  # type: ignore
from glclient import Credentials, Signer, Scheduler  # type: ignore
from pathlib import Path
from pyln import grpc as clnpb  # type: ignore
import secrets  # Make sure to use cryptographically sound randomness

GL_NOBODY_CRT = os.environ.get('GL_NOBODY_CRT')
GL_NOBODY_KEY = os.environ.get('GL_NOBODY_KEY')
NETWORK="regtest"
TEST_NODE_DATA_DIR_1="/tmp/gltests/node1"


def save_to_file(file_name: str, data: bytes) -> None:
    file_name = Path(TEST_NODE_DATA_DIR_1) / file_name
    os.makedirs(file_name.parent, exist_ok=True)    
    with open(file_name, "wb") as file:
        file.write(data)


def read_file(file_name: str) -> bytes:
    file_name = Path(TEST_NODE_DATA_DIR_1) / file_name
    with open(file_name, "rb") as file:
        return file.read()


def upgrade_device_certs_to_creds(scheduler: Scheduler, signer: Signer, device_creds_file_path: str):
    # ---8<--- [start: upgrade_device_certs_to_creds]
    device_creds = Credentials.from_path(device_creds_file_path)
    upgraded_device_creds = device_creds.upgrade(scheduler.inner, signer.inner)
    # ---8<--- [end: upgrade_device_certs_to_creds]
    return upgraded_device_creds


def create_seed() -> bytes:
    # ---8<--- [start: create_seed]
    rand = secrets.randbits(256).to_bytes(32, "big")  # 32 bytes of randomness

    # Seed phrase for user
    phrase = bip39.encode_bytes(rand)

    seed = bip39.phrase_to_seed(phrase)[:32]  # Only need the first 32 bytes

    # Store the seed on the filesystem, or secure configuration system
    save_to_file("hsm_secret", seed)

    # ---8<--- [end: create_seed]
    return seed


def load_developer_creds():
    # ---8<--- [start: dev_creds]
    developer_cert_path = os.environ.get('GL_NOBODY_CRT')
    developer_key_path = os.environ.get('GL_NOBODY_KEY')
    developer_cert = Path(developer_cert_path).open(mode="rb").read()
    developer_key = Path(developer_key_path).open(mode="rb").read()

    developer_creds = Credentials.nobody_with(developer_cert, developer_key)
    # ---8<--- [end: dev_creds]
    return developer_creds


def register_node(seed: bytes, developer_creds: Credentials):
    # ---8<--- [start: init_signer]
    signer = Signer(seed, NETWORK, developer_creds)
    # ---8<--- [end: init_signer]

    # ---8<--- [start: register_node]
    scheduler = Scheduler(NETWORK, developer_creds)

    # Passing in the signer is required because the client needs to prove
    # ownership of the `node_id`
    registration_response = scheduler.register(signer, invite_code=None)

    # ---8<--- [start: device_creds]
    device_creds = Credentials.from_bytes(registration_response.creds)
    save_to_file("credentials.gfs", device_creds.to_bytes())
    # ---8<--- [end: device_creds]
    # ---8<--- [end: register_node]
    return scheduler, device_creds, signer


def get_node(scheduler: Scheduler, device_creds: Credentials) -> dict:
    # ---8<--- [start: get_node]
    scheduler = scheduler.authenticate(device_creds)
    node = scheduler.node()
    # ---8<--- [end: get_node]
    return node


def start_node(device_creds_file_path: str) -> None:
    # ---8<--- [start: start_node]
    device_creds = Credentials.from_path(device_creds_file_path)
    scheduler = Scheduler(NETWORK, device_creds)

    node = scheduler.node()
    # ---8<--- [end: start_node]

    # ---8<--- [start: list_peers]
    getinfo_response = node.get_info()
    listpeers_response = node.list_peers()
    # ---8<--- [end: list_peers]

    # ---8<--- [start: start_signer]
    seed = read_file("hsm_secret")
    signer = Signer(seed, NETWORK, device_creds)

    signer.run_in_thread()
    # ---8<--- [end: start_signer]

    # ---8<--- [start: create_invoice]
    invoice_response = node.invoice(
        amount_msat=clnpb.AmountOrAny(amount=clnpb.Amount(msat=10000)),
        description="description".join(secrets.choice(string.ascii_letters + string.digits) for _ in range(10)),
        label="label".join(secrets.choice(string.ascii_letters + string.digits) for _ in range(10)),
    )
    # ---8<--- [end: create_invoice]
    return getinfo_response, listpeers_response, invoice_response


def recover_node(developer_cert: bytes, developer_key: bytes) -> None:
    # ---8<--- [start: recover_node]
    seed = read_file("hsm_secret")
    signer_creds = Credentials.nobody_with(developer_cert, developer_key)
    signer = Signer(seed, NETWORK, signer_creds)
    scheduler = Scheduler(NETWORK, signer_creds)
    recover_response = scheduler.recover(signer)
    # ---8<--- [end: recover_node]
    device_creds = Credentials.from_bytes(recover_response.creds)
    save_to_file("credentials.gfs", device_creds.to_bytes())
    return scheduler, device_creds, signer


def main():
    # Verify files exist
    if not GL_NOBODY_CRT or not os.path.exists(GL_NOBODY_CRT):
        print(f"Error: Developer certificate not found at {GL_NOBODY_CRT}")
        sys.exit(1)
        
    if not GL_NOBODY_KEY or not os.path.exists(GL_NOBODY_KEY):
        print(f"Error: Developer key not found at {GL_NOBODY_KEY}")
        sys.exit(1)
    
    #Create seed
    print("Creating seed...")
    seed = create_seed()
    
    print("Loading developer credentials...")
    developer_creds = load_developer_creds()

    # Register node
    print("Registering node...")
    scheduler, device_creds, signer = register_node(seed, developer_creds)
    print("Node Registered!")

    # Get GL node
    print("Getting node information...")
    get_node(scheduler, device_creds)
    
    # Print node's information to check
    getinfo_response, listpeers_response, invoice_response = start_node(TEST_NODE_DATA_DIR_1 + "/credentials.gfs")
    print("Node pubkey:", getinfo_response.id.hex())
    print("Peers list:", listpeers_response.peers)
    print("Invoice created:", invoice_response.bolt11)

    # Upgrade Certificates
    print("Upgrading certs...")
    upgrade_device_certs_to_creds(scheduler, signer, TEST_NODE_DATA_DIR_1 + "/credentials.gfs")
    
    # Recover the node
    print("Recovering node...")
    scheduler, device_creds, signer = recover_node(Path(GL_NOBODY_CRT).open(mode="rb").read(), Path(GL_NOBODY_KEY).open(mode="rb").read())
    print("Node Recovered!")

    # Print node's information to check
    getinfo_response, listpeers_response, invoice_response = start_node(TEST_NODE_DATA_DIR_1 + "/credentials.gfs")
    print("Node pubkey:", getinfo_response.id.hex())
    print("All steps completed successfully!")


if __name__ == "__main__":
    main()
