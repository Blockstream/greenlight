from dataclasses import dataclass, is_dataclass, asdict, field

import typing as t
import json
import time
import binascii

import glclient.glclient as native

import logging

logger = logging.getLogger(__name__)

class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if is_dataclass(o):
            return asdict(o)
        elif isinstance(o, NoParams):
            return dict()
        elif isinstance(o, type) and o.__name__ == "NoParams":
            return dict()
        return super().default(o)


class AsDataClassDescriptor:
    """Descriptor that allows to initialize a nested dataclass from a nested directory"""

    def __init__(self, *, cls):
        self._cls = cls

    def __set_name__(self, owner, name):
        self._name = f"_{name}"

    def __get__(self, obj, type):
        return getattr(obj, self._name, None)

    def __set__(self, obj, value):
        if isinstance(value, self._cls):
            setattr(obj, self._name, value)
        else:
            setattr(obj, self._name, self._cls(**value))


def _dump_json_bytes(object: t.Any) -> bytes:
    json_str: str = json.dumps(object, cls=EnhancedJSONEncoder)
    json_bytes: bytes = json_str.encode("utf-8")
    return json_bytes


@dataclass
class ProtocolList:
    protocols: t.List[int]


@dataclass
class Lsps1Options:
    minimum_channel_confirmations: t.Optional[int]
    minimum_onchain_payment_confirmations: t.Optional[int]
    supports_zero_channel_reserve: t.Optional[bool]
    min_onchain_payment_size_sat: t.Optional[int]
    max_channel_expiry_blocks: t.Optional[int]
    min_initial_client_balance_sat: t.Optional[int]
    min_initial_lsp_balance_sat: t.Optional[int]
    max_initial_client_balance_sat: t.Optional[int]
    min_channel_balance_sat: t.Optional[int]
    max_channel_balance_sat: t.Optional[int]


class NoParams:
    pass


class LspClient:
    def __init__(self, native: native.LspClient):
        self._native = native

    def _rpc_call(
        self,
        peer_id: str,
        method_name: str,
        param_json: bytes,
        json_rpc_id: t.Optional[str] = None,
    ) -> bytes:
        logger.debug("Request lsp to peer %s and method %s", peer_id, method_name)
        peer_id_bytes = bytes.fromhex(peer_id)
        if json_rpc_id is None:
            return self._native.rpc_call(peer_id_bytes, method_name, param_json)
        else:
            return self._native.rpc_call_with_json_rpc_id(
                peer_id_bytes, method_name, param_json, json_rpc_id=json_rpc_id
            )

    def list_lsp_servers(self) -> t.List[str]:
        return self._native.list_lsp_servers()

    def list_protocols(self, peer_id, json_rpc_id: t.Optional[str] = None) -> ProtocolList:
        json_bytes = _dump_json_bytes(NoParams)
        result = self._rpc_call(
            peer_id, "lsps0.list_protocols", json_bytes, json_rpc_id=json_rpc_id
        )
        response_dict = json.loads(result)
        return ProtocolList(**response_dict)
