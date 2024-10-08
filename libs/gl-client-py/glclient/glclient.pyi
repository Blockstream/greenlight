"""Stubs for the API exposed by the Rust `gl-client` library.

These refer to the API exposed by the Rust library, not the main
`glclient` python package. As such these mostly just concern the lower
level API that shuffles bytes back and forth. The `glclient` python
package adds a pythonic facade on top of this to improve usability.

"""

from typing import Optional, List
import glclient.glclient as native

class TlsConfig:
    def __init__(self) -> None: ...
    def with_ca_certificate(self, ca: bytes) -> "TlsConfig": ...
    def identity(self, cert_pem: bytes, key_pem: bytes) -> "TlsConfig": ...
    def identity_from_path(self, path: str) -> "TlsConfig": ...

class Credentials:
    def __init__(self) -> None: ...
    @staticmethod
    def nobody_with(cert: bytes, key: bytes) -> Credentials: ...
    @staticmethod
    def from_bytes(data: bytes) -> Credentials: ...
    @staticmethod
    def from_path(path: str) -> Credentials: ...
    @staticmethod
    def from_parts(cert: bytes, key: bytes, rune: str) -> Credentials: ...
    def node_id(self) -> bytes: ...
    def upgrade(self, scheduler: Scheduler, signer: Signer) -> Credentials: ...
    def to_bytes(self) -> bytes: ...
    def with_ca(self) -> Credentials: ...

class SignerHandle:
    def shutdown(self) -> None: ...

class Signer:
    def __init__(self, secret: bytes, network: str, creds: Credentials): ...
    def sign_challenge(self, challenge: bytes) -> bytes: ...
    def run_in_thread(self) -> SignerHandle: ...
    def run_in_foreground(self) -> None: ...
    def node_id(self) -> bytes: ...
    def version(self) -> str: ...
    def is_running(self) -> bool: ...
    def shutdown(self) -> None: ...
    def create_rune(
        self, restrictions: List[List[str]], rune: Optional[str] = None
    ) -> str: ...

class Scheduler:
    def __init__(self, network: str, creds: Optional[Credentials]): ...
    def register(self, signer: Signer, invite_code: Optional[str]) -> bytes: ...
    def recover(self, signer: Signer) -> bytes: ...
    def authenticate(self, creds: Credentials): ...
    def schedule(self) -> bytes: ...
    def node(self) -> bytes: ...
    def get_node_info(self, wait: bool) -> bytes: ...
    def export_node(self) -> bytes: ...
    def get_invite_codes(self) -> bytes: ...
    def add_outgoing_webhook(self, uri: str) -> bytes: ...
    def list_outgoing_webhooks(self) -> bytes: ...
    def delete_outgoing_webhook(self, webhook_id: int) -> bytes: ...
    def delete_outgoing_webhooks(self, webhook_ids: List[int]) -> bytes: ...
    def rotate_outgoing_webhook_secret(self, webhook_id: int) -> bytes: ...

class NewDeviceClient:
    def __init__(self, creds: Credentials, uri: Optional[str]): ...
    def pair_device(self, name: str, description: str, restrictions: str): ...

class AttestationDeviceClient:
    def __init__(self, creds: Credentials, uri: Optional[str]): ...
    def get_pairing_data(self, device_id: str) -> bytes: ...
    def approve_pairing(self, device_id: str, device_name: str, restrs: str):...
    def verify_pairing_data(self, data: bytes): ...

class Node:
    def __init__(
        self,
        node_id: bytes,
        grpc_uri: str,
        creds: Credentials,
    ) -> None: ...
    def stop(self) -> None: ...
    def call(self, method: str, request: bytes) -> bytes: ...
    def get_lsp_client(self) -> LspClient: ...
    def trampoline_pay(
        self,
        bolt11: str,
        trampoline_node_id: bytes,
        amount_msat: Optional[int] = None,
        label: Optional[str] = None,
    ) -> bytes: ...
    def configure(self, payload: bytes) -> None: ...

class LspClient:
    def rpc_call(self, peer_id: bytes, method: str, params: bytes) -> bytes: ...
    def rpc_call_with_json_rpc_id(
        self,
        peer_id: bytes,
        method: str,
        params: bytes,
        json_rpc_id: Optional[str] = None,
    ) -> bytes: ...
    def list_lsp_servers(self) -> List[str]: ...

def backup_decrypt_with_seed(encrypted: bytes, seed: bytes) -> bytes: ...
