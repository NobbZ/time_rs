// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Event store trait and implementations

use crate::error::Result;
use crate::event::{Event, StoredEvent};

/// A store for reading and writing events
///
/// The event store is responsible for persisting events and retrieving them in order.
///
/// # Example
///
/// <!-- This example uses no_run because it requires filesystem operations -->
/// ```no_run
/// use time_rs_sourcing::{EventStore, GixEventStore};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// enum MyEvent {
///     Created,
/// }
///
/// impl time_rs_sourcing::message::Message for MyEvent {
///     fn name(&self) -> &'static str {
///         "MyEvent"
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Initialize a new event store
/// let mut store = GixEventStore::init("./events")?;
///
/// // Append an event
/// let my_event = MyEvent::Created;
/// let commit_id = store.append(&my_event)?;
///
/// // Read all events
/// let events = store.read_all::<MyEvent>()?;
///
/// // Get latest commit
/// let latest = store.latest_commit()?;
/// # Ok(())
/// # }
/// ```
pub trait EventStore {
    /// Append a new event to the store
    ///
    /// Each event is stored in its own file with exactly one file per commit
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be serialized or stored
    fn append<E: Event>(&mut self, event: &E) -> Result<String>;

    /// Read all events from the store in chronological order
    ///
    /// # Errors
    ///
    /// Returns an error if events cannot be read or deserialized
    fn read_all<E: Event>(&self) -> Result<Vec<StoredEvent<E>>>;

    /// Read events from the store starting from a specific commit
    ///
    /// # Errors
    ///
    /// Returns an error if events cannot be read or deserialized
    fn read_from<E: Event>(&self, commit_id: &str) -> Result<Vec<StoredEvent<E>>>;

    /// Get the latest commit ID in the store
    ///
    /// # Errors
    ///
    /// Returns an error if the commit ID cannot be retrieved
    fn latest_commit(&self) -> Result<Option<String>>;
}
