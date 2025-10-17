"""Testing client-scheduler interactions
"""

import pytest
import glsdk
from gltesting.fixtures import *

def test_register(scheduler, clients):
    signer = glsdk.Signer("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
    
