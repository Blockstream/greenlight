from glclient import Signer
import logging
import time
import pytest


def test_init():
    device_cert = open('../device.crt', 'rb').read()
    device_key = open('../device-key.pem', 'rb').read()
    secret = open("../hsm_secret", 'rb').read()
    network = 'bitcoin'
    signer = Signer(secret, network).with_identity(device_cert, device_key)
    signer.run_in_thread()
    time.sleep(30)


def test_wrong_init():
    secret = open("../hsm_secret", 'rb').read()
    network = 'notanetwork'

    with pytest.raises(ValueError):
        signer = Signer(secret, network)
