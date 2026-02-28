//! Domain layer: entities, value objects, domain services, and domain events.

pub mod aggregates;
pub mod error;
pub mod value_objects;

pub use aggregates::*;
pub use error::DomainError;
pub use value_objects::*;
