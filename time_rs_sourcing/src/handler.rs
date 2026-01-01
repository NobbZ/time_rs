// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Event handler pattern for building read models and projections
//!
//! Event handlers subscribe to events and build read models, which can be
//! used for queries and reporting without affecting the write model.

use crate::error::Result;
use crate::event::{Event, StoredEvent};

/// An event handler processes events to build read models or trigger side effects
///
/// Event handlers are called after events are persisted and can be used to:
/// - Build read models / projections for queries
/// - Send notifications
/// - Update external systems
/// - Maintain materialized views
///
/// # Example
///
/// ```
/// use time_rs_sourcing::{EventHandler, Result, StoredEvent};
/// use serde::{Serialize, Deserialize};
/// use std::collections::HashMap;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// enum BankEvent {
///     AccountCreated { account_id: String, balance: i64 },
///     MoneyDeposited { account_id: String, amount: i64 },
/// }
///
/// impl time_rs_sourcing::message::Message for BankEvent {
///     fn name(&self) -> &'static str {
///         "BankEvent"
///     }
/// }
///
/// // Read model for account balances
/// struct AccountBalanceReadModel {
///     balances: HashMap<String, i64>,
/// }
///
/// impl EventHandler for AccountBalanceReadModel {
///     type Event = BankEvent;
///
///     fn handle(&mut self, event: &StoredEvent<Self::Event>) -> Result<()> {
///         match &event.event {
///             BankEvent::AccountCreated { account_id, balance } => {
///                 self.balances.insert(account_id.clone(), *balance);
///             }
///             BankEvent::MoneyDeposited { account_id, amount } => {
///                 if let Some(balance) = self.balances.get_mut(account_id) {
///                     *balance += amount;
///                 }
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait EventHandler {
    /// The type of events this handler processes
    type Event: Event;

    /// Handle a single event
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be processed
    fn handle(&mut self, event: &StoredEvent<Self::Event>) -> Result<()>;

    /// Handle multiple events in sequence
    ///
    /// # Errors
    ///
    /// Returns an error if any event cannot be processed
    fn handle_many(&mut self, events: &[StoredEvent<Self::Event>]) -> Result<()> {
        for event in events {
            self.handle(event)?;
        }
        Ok(())
    }
}

/// A dispatcher that routes events to multiple handlers
///
/// # Example
///
/// ```
/// use time_rs_sourcing::{EventHandler, EventDispatcher, Result, StoredEvent};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// enum MyEvent {
///     Something,
/// }
///
/// impl time_rs_sourcing::message::Message for MyEvent {
///     fn name(&self) -> &'static str {
///         "MyEvent"
///     }
/// }
///
/// struct LoggingHandler;
///
/// impl EventHandler for LoggingHandler {
///     type Event = MyEvent;
///
///     fn handle(&mut self, event: &StoredEvent<Self::Event>) -> Result<()> {
///         println!("Event logged: {:?}", event.event);
///         Ok(())
///     }
/// }
///
/// struct MetricsHandler {
///     count: usize,
/// }
///
/// impl EventHandler for MetricsHandler {
///     type Event = MyEvent;
///
///     fn handle(&mut self, _event: &StoredEvent<Self::Event>) -> Result<()> {
///         self.count += 1;
///         Ok(())
///     }
/// }
///
/// let mut dispatcher = EventDispatcher::new();
/// dispatcher.register(Box::new(LoggingHandler));
/// dispatcher.register(Box::new(MetricsHandler { count: 0 }));
/// ```
pub struct EventDispatcher<E: Event> {
    handlers: Vec<Box<dyn EventHandler<Event = E>>>,
}

impl<E: Event> EventDispatcher<E> {
    /// Create a new event dispatcher
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register an event handler
    pub fn register(&mut self, handler: Box<dyn EventHandler<Event = E>>) {
        self.handlers.push(handler);
    }

    /// Dispatch an event to all registered handlers
    ///
    /// # Errors
    ///
    /// Returns an error if any handler fails to process the event
    pub fn dispatch(&mut self, event: &StoredEvent<E>) -> Result<()> {
        for handler in &mut self.handlers {
            handler.handle(event)?;
        }
        Ok(())
    }

    /// Dispatch multiple events to all registered handlers
    ///
    /// # Errors
    ///
    /// Returns an error if any handler fails to process any event
    pub fn dispatch_many(&mut self, events: &[StoredEvent<E>]) -> Result<()> {
        for event in events {
            self.dispatch(event)?;
        }
        Ok(())
    }
}

impl<E: Event> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use crate::event::EventMetadata;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl crate::message::Message for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    struct CountingHandler {
        count: usize,
        sum: i32,
    }

    impl EventHandler for CountingHandler {
        type Event = TestEvent;

        fn handle(&mut self, event: &StoredEvent<Self::Event>) -> Result<()> {
            self.count += 1;
            self.sum += event.event.value;
            Ok(())
        }
    }

    #[test]
    fn test_event_handler_processes_events() {
        let mut handler = CountingHandler { count: 0, sum: 0 };

        let event = StoredEvent {
            event: TestEvent { value: 10 },
            metadata: EventMetadata {
                commit_id: "abc".to_string(),
                timestamp: 123,
                author: "test".to_string(),
                file_path: "test.json".to_string(),
            },
        };

        handler.handle(&event).unwrap();
        assert_eq!(handler.count, 1);
        assert_eq!(handler.sum, 10);
    }

    #[test]
    fn test_event_handler_processes_many_events() {
        let mut handler = CountingHandler { count: 0, sum: 0 };

        let events = vec![
            StoredEvent {
                event: TestEvent { value: 10 },
                metadata: EventMetadata {
                    commit_id: "abc".to_string(),
                    timestamp: 123,
                    author: "test".to_string(),
                    file_path: "test.json".to_string(),
                },
            },
            StoredEvent {
                event: TestEvent { value: 20 },
                metadata: EventMetadata {
                    commit_id: "def".to_string(),
                    timestamp: 124,
                    author: "test".to_string(),
                    file_path: "test2.json".to_string(),
                },
            },
        ];

        handler.handle_many(&events).unwrap();
        assert_eq!(handler.count, 2);
        assert_eq!(handler.sum, 30);
    }

    #[test]
    fn test_dispatcher_routes_to_multiple_handlers() {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Box::new(CountingHandler { count: 0, sum: 0 }));

        let event = StoredEvent {
            event: TestEvent { value: 15 },
            metadata: EventMetadata {
                commit_id: "abc".to_string(),
                timestamp: 123,
                author: "test".to_string(),
                file_path: "test.json".to_string(),
            },
        };

        dispatcher.dispatch(&event).unwrap();
    }

    #[test]
    fn test_dispatcher_dispatch_many() {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Box::new(CountingHandler { count: 0, sum: 0 }));

        let events = vec![
            StoredEvent {
                event: TestEvent { value: 5 },
                metadata: EventMetadata {
                    commit_id: "abc".to_string(),
                    timestamp: 123,
                    author: "test".to_string(),
                    file_path: "test.json".to_string(),
                },
            },
            StoredEvent {
                event: TestEvent { value: 10 },
                metadata: EventMetadata {
                    commit_id: "def".to_string(),
                    timestamp: 124,
                    author: "test".to_string(),
                    file_path: "test2.json".to_string(),
                },
            },
        ];

        dispatcher.dispatch_many(&events).unwrap();
    }
}
