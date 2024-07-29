#!/usr/bin/env python3

from pyln.client import Plugin
from binascii import unhexlify
from pyln.proto.primitives import varint_decode
from io import BytesIO
import time

INVOICE_TYPE = 33001
AMOUNT_TYPE = 33003

plugin = Plugin(
    dynamic=False,
    init_features=1 << 427,
)

plugin.check_invoice = None
plugin.check_amount = None
plugin.payment_key = None
plugin.waitfor = None


@plugin.hook("htlc_accepted")
def on_htlc_accepted(htlc, onion, plugin, **kwargs):
    plugin.log(f"got onion {onion}")

    payment_metadata = unhexlify(onion["payment_metadata"])
    payment_metadata = BytesIO(payment_metadata)

    invoice_type = varint_decode(payment_metadata)
    invoice_length = varint_decode(payment_metadata)
    invoice_value = payment_metadata.read(invoice_length)
    assert invoice_type == INVOICE_TYPE

    if plugin.waitfor is not None:
        time.sleep(plugin.waitfor)

    if plugin.check_invoice is not None:
        plugin.log(
            f"check invoice {invoice_value.decode('utf-8')} matches {plugin.check_invoice}"
        )
        assert invoice_value.decode("utf-8") == plugin.check_invoice

    amount_msat_type = varint_decode(payment_metadata)
    amount_msat_length = varint_decode(payment_metadata)
    amount_msat_value = payment_metadata.read(amount_msat_length)
    assert amount_msat_type == AMOUNT_TYPE

    if plugin.check_amount is not None:
        plugin.log(
            f"check amount_msat {int.from_bytes(amount_msat_value, 'big')} matches {plugin.check_amount}"
        )
        assert int.from_bytes(amount_msat_value, "big") == plugin.check_amount

    plugin.log(f"got invoice={invoice_value.decode('utf-8')}")
    plugin.log(f"got amount_msat={int.from_bytes(amount_msat_value, 'big')}")

    if plugin.payment_key is not None:
        plugin.log(f"resolving with payment_key {plugin.payment_key}")
        return {
            "result": "resolve",
            "payment_key": plugin.payment_key,
        }

    replacement = onion["payload"][6:102]
    plugin.log(f"replace onion payload with {replacement}")
    return {"result": "continue", "payload": replacement}


@plugin.method("setpaymentkey")
def setpaymentkey(plugin, payment_key):
    """Sets the payment_key to resolve an htlc"""
    plugin.payment_key = payment_key


@plugin.method("setcheckinvoice")
def setcheckinvoice(plugin, invoice):
    """Sets an invoice check"""
    plugin.check_invoice = invoice


@plugin.method("setcheckamount")
def setcheckamount(plugin, amount_msat):
    """Sets an amount check"""
    plugin.check_amount = amount_msat


@plugin.method("unsetchecks")
def unsetchecks(plugin):
    """Unsets all checks"""
    plugin.check_invoice = None
    plugin.check_amount = None


@plugin.method("waitfor")
def waitfor(plugin, duration_sec):
    """Waits for duration_sec before continuing with completion"""
    plugin.waitfor = duration_sec


plugin.run()
