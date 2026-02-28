//! Infrastructure layer: database, external clients, adapters, and brokers.

pub mod adapters;
pub mod outbox;
pub mod persistence;
pub mod ports;

pub use outbox::publisher::{OutboxEvent, OutboxPublisher, PublishTransport};
pub use ports::*;
