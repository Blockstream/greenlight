//! Event system for gl-plugin with support for internal extensions.
//!
//! This module provides a generic `Event<I>` enum that can be extended
//! by downstream crates (like gl-plugin-internal) with custom internal
//! event types, while keeping the core events defined here.
//!
//! # Architecture
//!
//! The event system has two layers:
//!
//! - **Internal events** (`Event::Internal(I)`) - Type-erased events for
//!   communication within the plugin process. Not exposed to clients.
//! - **Public events** (`Event::IncomingPayment`, etc.) - Automatically
//!   converted to `NodeEvent` protobufs and streamed to gl-sdk clients
//!   via `StreamNodeEvents`.
//!
//! The [`EventBus`] uses `tokio::sync::broadcast` so multiple subscribers
//! can receive the same events. Events are not persisted - if a client
//! disconnects, it won't receive missed events.
//!
//! # Examples
//!
//! In these examples, we define a type alias for the event type to avoid
//! repeating the generic parameter. This is the recommended pattern:
//!
//! ```ignore
//! use gl_plugin::events::Event;
//!
//! // In gl-plugin (no internal events):
//! type PluginEvent = Event<()>;
//!
//! // In gl-plugin-internal (with internal events):
//! #[derive(Clone, Debug)]
//! pub enum InternalPayload {
//!     NodeMeta { node_id: Vec<u8>, version: String },
//!     Shutdown { reason: String },
//! }
//! type PluginEvent = Event<InternalPayload>;
//! ```
//!
//! ## Publishing an Internal Event
//!
//! Internal events are for communication within the plugin system and
//! are NOT exposed to clients. Use the `Internal` variant with your payload:
//!
//! ```ignore
//! use gl_plugin::events::{Event, EventBus};
//!
//! // Define your internal payload type
//! #[derive(Clone, Debug)]
//! pub enum InternalPayload {
//!     NodeMeta { node_id: Vec<u8>, version: String },
//!     Shutdown { reason: String },
//! }
//!
//! // Define a type alias for convenience
//! type PluginEvent = Event<InternalPayload>;
//!
//! fn publish_internal_event(bus: &EventBus) {
//!     let payload = InternalPayload::NodeMeta {
//!         node_id: vec![0x02; 33],
//!         version: "v25.12".to_string(),
//!     };
//!     bus.publish(PluginEvent::Internal(payload));
//! }
//! ```
//!
//! ## Subscribing to Internal Events
//!
//! Use the [`ErasedEventExt`] trait to downcast internal payloads:
//!
//! ```ignore
//! use gl_plugin::events::{EventBus, ErasedEventExt};
//!
//! async fn subscribe_to_internal_events(bus: &EventBus) {
//!     let mut rx = bus.subscribe();
//!
//!     while let Ok(event) = rx.recv().await {
//!         if let Some(payload) = event.downcast_internal::<InternalPayload>() {
//!             match payload {
//!                 InternalPayload::NodeMeta { node_id, version } => {
//!                     println!("Node {} running {}", hex::encode(node_id), version);
//!                 }
//!                 InternalPayload::Shutdown { reason } => {
//!                     println!("Shutting down: {}", reason);
//!                     break;
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Publishing a Public Event
//!
//! Public events like `IncomingPayment` are automatically converted to
//! `NodeEvent` protobufs and streamed to clients via `StreamNodeEvents`.
//! Using the type alias, the same code works regardless of whether you
//! have internal events or not:
//!
//! ```ignore
//! use gl_plugin::events::{Event, EventBus};
//! use gl_plugin::pb;
//!
//! // Use () if you don't need internal events, or your InternalPayload type
//! type PluginEvent = Event<()>;
//!
//! fn on_invoice_payment(bus: &EventBus, payment_hash: Vec<u8>,
//!                       bolt11: String, preimage: Vec<u8>, amount_msat: u64) {
//!     let payment = pb::IncomingPayment {
//!         details: Some(pb::incoming_payment::Details::Offchain(pb::OffChain {
//!             payment_hash,
//!             bolt11,
//!             preimage,
//!             label: "my-invoice".to_string(),
//!             amount: Some(pb::Amount {
//!                 unit: Some(pb::amount::Unit::Millisatoshi(amount_msat)),
//!             }),
//!             extratlvs: vec![],
//!         })),
//!     };
//!
//!     // This event will be streamed to gl-sdk clients as NodeEvent::InvoicePaid
//!     bus.publish(PluginEvent::IncomingPayment(payment));
//! }
//! ```
//!
//! ## Subscribing to Public Events (Server-side)
//!
//! When subscribing, you receive `ErasedEvent` which works with any
//! internal type. Pattern match on the public variants directly:
//!
//! ```ignore
//! use gl_plugin::events::{Event, EventBus, ErasedEvent};
//!
//! async fn monitor_payments(bus: &EventBus) {
//!     let mut rx = bus.subscribe();
//!
//!     while let Ok(event) = rx.recv().await {
//!         // ErasedEvent can be matched on public variants directly
//!         match &event {
//!             Event::IncomingPayment(p) => {
//!                 println!("Payment received!");
//!             }
//!             Event::CustomMsg(msg) => {
//!                 println!("Custom message from peer");
//!             }
//!             Event::Stop(_) => {
//!                 println!("Plugin stopping");
//!                 break;
//!             }
//!             Event::Internal(_) => {
//!                 // Use downcast_internal::<T>() to access typed payload
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! ## Client-side: Consuming Events via gl-sdk (Python)
//!
//! ```python
//! # Stream events from a Greenlight node
//! for event in node.stream_node_events():
//!     if event.HasField("invoice_paid"):
//!         paid = event.invoice_paid
//!         print(f"Invoice paid: {paid.amount_msat} msat")
//!         print(f"  hash: {paid.payment_hash.hex()}")
//! ```
//!
//! ## Client-side: Consuming Events via gl-sdk (Rust)
//!
//! ```ignore
//! use gl_client::pb::{NodeEventsRequest, node_event::Event};
//!
//! async fn stream_events(mut client: NodeClient) -> Result<()> {
//!     let stream = client
//!         .stream_node_events(NodeEventsRequest {})
//!         .await?
//!         .into_inner();
//!
//!     while let Some(event) = stream.message().await? {
//!         match event.event {
//!             Some(Event::InvoicePaid(paid)) => {
//!                 println!("Invoice paid: {} msat", paid.amount_msat);
//!             }
//!             _ => {}
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use crate::pb;
use crate::stager;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::broadcast;

/// An event that can be observed during the operation of the plugin.
///
/// The type parameter `I` allows downstream crates to extend the event
/// system with internal-only event types. In gl-plugin, `I` defaults to
/// `()` (no internal events). In gl-plugin-internal, `I` is set to
/// `InternalPayload` containing events like `NodeMeta`, `Shutdown`, etc.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Event<I = ()> {
    /// The plugin is stopping.
    Stop(Arc<stager::Stage>),

    /// A gRPC call was made. The string is the URI of the request.
    RpcCall(String),

    /// An incoming payment was received (invoice paid).
    IncomingPayment(pb::IncomingPayment),

    /// A custom message was received from a peer.
    CustomMsg(pb::Custommsg),

    /// Internal events from gl-plugin-internal or other extensions.
    /// This variant is not used when `I = ()`.
    Internal(I),
}

impl<I> Event<I> {
    /// Transform the internal payload type using a mapping function.
    pub fn map_internal<J, F>(self, f: F) -> Event<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Event::Stop(s) => Event::Stop(s),
            Event::RpcCall(r) => Event::RpcCall(r),
            Event::IncomingPayment(p) => Event::IncomingPayment(p),
            Event::CustomMsg(m) => Event::CustomMsg(m),
            Event::Internal(i) => Event::Internal(f(i)),
        }
    }

    /// Try to transform the internal payload, returning None if the
    /// transformation fails.
    pub fn try_map_internal<J, F>(self, f: F) -> Option<Event<J>>
    where
        F: FnOnce(I) -> Option<J>,
    {
        match self {
            Event::Stop(s) => Some(Event::Stop(s)),
            Event::RpcCall(r) => Some(Event::RpcCall(r)),
            Event::IncomingPayment(p) => Some(Event::IncomingPayment(p)),
            Event::CustomMsg(m) => Some(Event::CustomMsg(m)),
            Event::Internal(i) => f(i).map(Event::Internal),
        }
    }
}

/// Type alias for the erased internal event type used by EventBus.
/// Uses Arc for clonability required by broadcast channel.
pub type ErasedInternal = Arc<dyn Any + Send + Sync>;

/// Type alias for events with type-erased internal payload.
pub type ErasedEvent = Event<ErasedInternal>;

/// An event bus that supports multiple subscribers and type-erased
/// internal events.
///
/// The bus internally stores `Event<Box<dyn Any + Send + Sync>>` to
/// allow publishing events with any internal payload type. Subscribers
/// can then downcast back to their expected type.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<ErasedEvent>,
}

impl EventBus {
    /// Create a new event bus with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event with a typed internal payload.
    ///
    /// The internal payload is type-erased before being sent on the bus.
    pub fn publish<I>(&self, event: Event<I>)
    where
        I: Send + Sync + 'static,
    {
        let erased = event.map_internal(|i| Arc::new(i) as ErasedInternal);
        // Ignore error if no subscribers
        let _ = self.sender.send(erased);
    }

    /// Subscribe to receive all events with type-erased internal payloads.
    pub fn subscribe(&self) -> broadcast::Receiver<ErasedEvent> {
        self.sender.subscribe()
    }

    /// Get a clone of the sender for sharing across tasks.
    pub fn sender(&self) -> broadcast::Sender<ErasedEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(16)
    }
}

/// Extension trait for working with erased events.
pub trait ErasedEventExt {
    /// Try to downcast the internal payload to a specific type.
    ///
    /// Returns `Some(&T)` if the event is `Event::Internal` and the
    /// payload can be downcast to `T`, otherwise returns `None`.
    fn downcast_internal<T: 'static>(&self) -> Option<&T>;

    /// Try to convert this erased event back to a typed event.
    ///
    /// Returns `Some(Event<I>)` if the internal payload (if present)
    /// can be downcast to `I`, otherwise returns `None`.
    fn try_into_typed<I: Clone + 'static>(&self) -> Option<Event<I>>;
}

impl ErasedEventExt for ErasedEvent {
    fn downcast_internal<T: 'static>(&self) -> Option<&T> {
        match self {
            Event::Internal(any) => any.downcast_ref::<T>(),
            _ => None,
        }
    }

    fn try_into_typed<I: Clone + 'static>(&self) -> Option<Event<I>> {
        match self {
            Event::Stop(s) => Some(Event::Stop(s.clone())),
            Event::RpcCall(r) => Some(Event::RpcCall(r.clone())),
            Event::IncomingPayment(p) => Some(Event::IncomingPayment(p.clone())),
            Event::CustomMsg(m) => Some(Event::CustomMsg(m.clone())),
            Event::Internal(any) => any.downcast_ref::<I>().cloned().map(Event::Internal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestInternal {
        value: i32,
    }

    #[test]
    fn test_event_map_internal() {
        let event: Event<i32> = Event::Internal(42);
        let mapped: Event<String> = event.map_internal(|i| i.to_string());
        match mapped {
            Event::Internal(s) => assert_eq!(s, "42"),
            _ => panic!("Expected Internal variant"),
        }
    }

    #[test]
    fn test_event_map_internal_preserves_other_variants() {
        let event: Event<i32> = Event::RpcCall("test".to_string());
        let mapped: Event<String> = event.map_internal(|i| i.to_string());
        match mapped {
            Event::RpcCall(s) => assert_eq!(s, "test"),
            _ => panic!("Expected RpcCall variant"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(8);
        let mut rx = bus.subscribe();

        bus.publish(Event::<()>::RpcCall("test_method".to_string()));

        let received = rx.recv().await.unwrap();
        match received {
            Event::RpcCall(method) => assert_eq!(method, "test_method"),
            _ => panic!("Expected RpcCall"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_with_internal_payload() {
        let bus = EventBus::new(8);
        let mut rx = bus.subscribe();

        let internal = TestInternal { value: 123 };
        bus.publish(Event::Internal(internal.clone()));

        let received = rx.recv().await.unwrap();
        let downcasted = received.downcast_internal::<TestInternal>();
        assert_eq!(downcasted, Some(&internal));
    }

    #[test]
    fn test_erased_event_try_into_typed() {
        let bus = EventBus::new(8);

        // Publish a typed event
        let internal = TestInternal { value: 456 };
        let event = Event::Internal(internal.clone());
        let erased = event.map_internal(|i| Arc::new(i) as ErasedInternal);

        // Try to convert back
        let typed: Option<Event<TestInternal>> = erased.try_into_typed();
        assert!(typed.is_some());
        match typed.unwrap() {
            Event::Internal(t) => assert_eq!(t, internal),
            _ => panic!("Expected Internal variant"),
        }

        // Suppress unused variable warning
        let _ = bus;
    }
}
