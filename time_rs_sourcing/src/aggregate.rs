// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Aggregate trait for event sourcing

use crate::error::Result;
use crate::event::Event;

/// An aggregate is a stateful entity that can be reconstructed from events
///
/// Aggregates process events to build up their state through event replay
pub trait Aggregate: Sized + Default {
    /// The type of events this aggregate processes
    type Event: Event;

    /// Apply an event to this aggregate, updating its state
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be applied
    fn apply(&mut self, event: &Self::Event) -> Result<()>;

    /// Replay a sequence of events to reconstruct the aggregate's state
    ///
    /// # Errors
    ///
    /// Returns an error if any event cannot be applied
    fn replay(events: impl IntoIterator<Item = Self::Event>) -> Result<Self> {
        let mut aggregate = Self::default();
        for event in events {
            aggregate.apply(&event)?;
        }
        Ok(aggregate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use time_rs_derive::Message;

    #[derive(Debug, Clone, PartialEq, Eq, Message, Serialize, Deserialize)]
    enum CounterEvent {
        Incremented,
        Decremented,
        Reset,
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Counter {
        count: i32,
    }

    impl Aggregate for Counter {
        type Event = CounterEvent;

        fn apply(&mut self, event: &Self::Event) -> Result<()> {
            match event {
                CounterEvent::Incremented => self.count += 1,
                CounterEvent::Decremented => self.count -= 1,
                CounterEvent::Reset => self.count = 0,
            }
            Ok(())
        }
    }

    #[rstest]
    fn aggregate_starts_with_default_state() {
        let counter = Counter::default();
        assert_eq!(counter.count, 0);
    }

    #[rstest]
    fn can_apply_single_event() {
        let mut counter = Counter::default();
        counter.apply(&CounterEvent::Incremented).unwrap();
        assert_eq!(counter.count, 1);
    }

    #[rstest]
    fn can_replay_multiple_events() {
        let events = vec![
            CounterEvent::Incremented,
            CounterEvent::Incremented,
            CounterEvent::Decremented,
            CounterEvent::Incremented,
        ];

        let counter = Counter::replay(events).unwrap();
        assert_eq!(counter.count, 2);
    }

    #[rstest]
    fn replay_produces_correct_final_state() {
        let events = vec![
            CounterEvent::Incremented,
            CounterEvent::Incremented,
            CounterEvent::Incremented,
            CounterEvent::Reset,
            CounterEvent::Incremented,
        ];

        let counter = Counter::replay(events).unwrap();
        assert_eq!(counter.count, 1);
    }
}
