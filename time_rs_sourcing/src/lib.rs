// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! A specialised event sourced backend that can deal with the events beeing through git
//!
//! This crate provides a fully-fledged event sourcing system that uses git (via gix) as an event store.
//! Each event is stored in its own file, with exactly one file per commit.
//!
//! # Core Concepts
//!
//! - **Events**: Immutable facts that represent state changes
//! - **Event Store**: Persistence layer for events using git
//! - **Aggregates**: Stateful entities reconstructed by replaying events
//!
//! # Example
//!
//! ```rust,ignore
//! use time_rs_sourcing::{GixEventStore, EventStore, Aggregate};
//! use serde::{Serialize, Deserialize};
//! use time_rs_sourcing::message::Message;
//!
//! #[derive(Debug, Clone, Message, Serialize, Deserialize)]
//! enum MyEvent {
//!     Created,
//!     Updated,
//! }
//!
//! // Create or open an event store
//! let mut store = GixEventStore::init("./my-events").unwrap();
//!
//! // Append events (one file per commit)
//! store.append(&MyEvent::Created).unwrap();
//! store.append(&MyEvent::Updated).unwrap();
//!
//! // Read all events
//! let events = store.read_all::<MyEvent>().unwrap();
//! ```

pub mod aggregate;
pub mod error;
pub mod event;
pub mod gix_store;
pub mod message;
pub mod store;

// Re-export commonly used types
pub use aggregate::Aggregate;
pub use error::{EventSourcingError, Result};
pub use event::{Event, EventMetadata, StoredEvent};
pub use gix_store::GixEventStore;
pub use store::EventStore;
