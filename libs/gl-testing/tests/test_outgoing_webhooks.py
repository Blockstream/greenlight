"""Tests the outgoing webhook methods on the mock scheduler"""

from gltesting.fixtures import *
from glclient import scheduler_pb2 as schedpb
import asyncio

def test_add_outgoing_webhook(scheduler, clients):
    c = clients.new()
    r = c.register(configure=True)
    n = c.find_node()
    
    asyncio.run(scheduler.add_outgoing_webhook(schedpb.AddOutgoingWebhookRequest(**{
      "node_id": n.node_id,
      "uri": "https://blockstream.com"
    })))
    
    l = asyncio.run(scheduler.list_outgoing_webhooks(schedpb.ListOutgoingWebhooksRequest(**{
      "node_id": n.node_id
    })))
    
    assert len(l.outgoing_webhooks) == 1
    
def test_rotate_outgoing_webhook(scheduler, clients):
    c = clients.new()
    r = c.register(configure=True)
    n = c.find_node()
    
    add_webhook_response = asyncio.run(scheduler.add_outgoing_webhook(schedpb.AddOutgoingWebhookRequest(**{
      "node_id": n.node_id,
      "uri": "https://blockstream.com"
    })))
    
    l = asyncio.run(scheduler.list_outgoing_webhooks(schedpb.ListOutgoingWebhooksRequest(**{
      "node_id": n.node_id
    })))
    
    
    rotate_response = asyncio.run(scheduler.rotate_outgoing_webhook_secret(schedpb.RotateOutgoingWebhookSecretRequest(**{
      "node_id": n.node_id,
      "webhook_id": l.outgoing_webhooks[0].id
    })))
    
    assert add_webhook_response.secret != rotate_response.secret
    
def test_delete_outgoing_webhook(scheduler, clients):
    c = clients.new()
    r = c.register(configure=True)
    n = c.find_node()
    
    add_webhook_response = asyncio.run(scheduler.add_outgoing_webhook(schedpb.AddOutgoingWebhookRequest(**{
      "node_id": n.node_id,
      "uri": "https://blockstream.com"
    })))
    
    l = asyncio.run(scheduler.list_outgoing_webhooks(schedpb.ListOutgoingWebhooksRequest(**{
      "node_id": n.node_id
    })))
    
    assert len(l.outgoing_webhooks) == 1
    
    rotate_response = asyncio.run(scheduler.delete_outgoing_webhooks(schedpb.DeleteOutgoingWebhooksRequest(**{
      "node_id": n.node_id,
      "ids": [l.outgoing_webhooks[0].id]
    })))
    
    l = asyncio.run(scheduler.list_outgoing_webhooks(schedpb.ListOutgoingWebhooksRequest(**{
      "node_id": n.node_id
    })))
    
    assert len(l.outgoing_webhooks) == 0