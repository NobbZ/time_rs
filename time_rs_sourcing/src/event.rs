// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Event trait and related types for event sourcing

use serde::{Deserialize, Serialize};

use crate::message::Message;

/// An [`Event`] is a [`Message`] that represents something that has happened in the past
///
/// Events are immutable facts that describe state changes in the system
pub trait Event: Message + Serialize + for<'de> Deserialize<'de> {}

/// Automatically implement Event for any type that implements the required traits
impl<T> Event for T where T: Message + Serialize + for<'de> Deserialize<'de> {}

/// Metadata about when and how an event was stored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventMetadata {
    /// The commit hash where this event is stored
    pub commit_id: String,
    /// The timestamp when the event was committed
    pub timestamp: i64,
    /// The author who committed the event
    pub author: String,
    /// The file path where the event is stored
    pub file_path: String,
}

/// An event with its associated metadata
#[derive(Debug, Clone, Serialize)]
#[serde(bound(serialize = "E: Event"))]
pub struct StoredEvent<E>
where
    E: Event,
{
    /// The event data
    pub event: E,
    /// Metadata about the storage
    pub metadata: EventMetadata,
}

impl<'de, E> Deserialize<'de> for StoredEvent<E>
where
    E: Event,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StoredEventHelper<E> {
            event: E,
            metadata: EventMetadata,
        }

        let helper = StoredEventHelper::<E>::deserialize(deserializer)?;
        Ok(StoredEvent {
            event: helper.event,
            metadata: helper.metadata,
        })
    }
}
