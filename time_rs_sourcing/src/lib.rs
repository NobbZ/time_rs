// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! A specialised event sourced backend that can deal with the events being through git
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
//! <!-- This example uses no_run because it requires filesystem operations -->
//! ```no_run
//! use time_rs_sourcing::{GixEventStore, EventStore, Aggregate};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! enum MyEvent {
//!     Created,
//!     Updated,
//! }
//!
//! impl time_rs_sourcing::message::Message for MyEvent {
//!     fn name(&self) -> &'static str {
//!         "MyEvent"
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create or open an event store
//! let mut store = GixEventStore::init("./my-events")?;
//!
//! // Append events (one file per commit)
//! store.append(&MyEvent::Created)?;
//! store.append(&MyEvent::Updated)?;
//!
//! // Read all events
//! let events = store.read_all::<MyEvent>()?;
//! # Ok(())
//! # }
//! ```

pub mod aggregate;
pub mod command;
pub mod error;
pub mod event;
pub mod gix_store;
pub mod handler;
pub mod message;
pub mod store;

// Re-export commonly used types
pub use aggregate::Aggregate;
pub use command::{Command, CommandHandler};
pub use error::{EventSourcingError, Result};
pub use event::{Event, EventMetadata, StoredEvent};
pub use gix_store::GixEventStore;
pub use handler::{EventDispatcher, EventHandler};
pub use store::EventStore;
