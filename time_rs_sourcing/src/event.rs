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
///
/// # Examples
///
/// ```
/// use time_rs_sourcing::EventMetadata;
///
/// let metadata = EventMetadata {
///     commit_id: "abc123".to_string(),
///     timestamp: 1_234_567_890,
///     author: "Event Store".to_string(),
///     file_path: "events/event.json".to_string(),
/// };
///
/// assert_eq!(metadata.commit_id, "abc123");
/// assert_eq!(metadata.timestamp, 1_234_567_890);
/// ```
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
        Ok(Self {
            event: helper.event,
            metadata: helper.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEvent {
        data: String,
    }

    impl crate::message::Message for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata {
            commit_id: "abc123".to_string(),
            timestamp: 1_234_567_890,
            author: "Test Author".to_string(),
            file_path: "events/test.json".to_string(),
        };

        assert_eq!(metadata.commit_id, "abc123");
        assert_eq!(metadata.timestamp, 1_234_567_890);
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.file_path, "events/test.json");
    }

    #[test]
    fn test_event_metadata_equality() {
        let metadata1 = EventMetadata {
            commit_id: "abc123".to_string(),
            timestamp: 1_234_567_890,
            author: "Author".to_string(),
            file_path: "events/test.json".to_string(),
        };

        let metadata2 = metadata1.clone();

        assert_eq!(metadata1, metadata2);
    }

    #[test]
    fn test_stored_event_creation() {
        let event = TestEvent {
            data: "test data".to_string(),
        };

        let metadata = EventMetadata {
            commit_id: "abc123".to_string(),
            timestamp: 1_234_567_890,
            author: "Author".to_string(),
            file_path: "events/test.json".to_string(),
        };

        let stored = StoredEvent {
            event: event.clone(),
            metadata: metadata.clone(),
        };

        assert_eq!(stored.event, event);
        assert_eq!(stored.metadata, metadata);
    }

    #[test]
    fn test_stored_event_serialization() {
        let event = TestEvent {
            data: "test data".to_string(),
        };

        let metadata = EventMetadata {
            commit_id: "abc123".to_string(),
            timestamp: 1_234_567_890,
            author: "Author".to_string(),
            file_path: "events/test.json".to_string(),
        };

        let stored = StoredEvent { event, metadata };

        let json = serde_json::to_string(&stored).unwrap();
        let deserialized: StoredEvent<TestEvent> = serde_json::from_str(&json).unwrap();

        assert_eq!(stored.event, deserialized.event);
        assert_eq!(stored.metadata, deserialized.metadata);
    }

    #[test]
    fn test_event_trait_implementation() {
        // TestEvent should implement Event trait
        fn assert_event<T: Event>() {}
        assert_event::<TestEvent>();
    }
}
