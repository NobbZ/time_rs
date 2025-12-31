// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Example demonstrating the event sourcing system with a simple counter

use serde::{Deserialize, Serialize};
use time_rs_sourcing::{Aggregate, EventStore, GixEventStore};

// Define your events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum CounterEvent {
    Incremented,
    Decremented,
    Reset,
}

// Implement Message trait (needed for Event)
impl time_rs_sourcing::message::Message for CounterEvent {
    fn name(&self) -> &'static str {
        "examples::counter::CounterEvent"
    }
}

// Define your aggregate
#[derive(Debug, Default, PartialEq, Eq)]
struct Counter {
    count: i32,
}

// Implement Aggregate trait
impl Aggregate for Counter {
    type Event = CounterEvent;

    fn apply(&mut self, event: &Self::Event) -> time_rs_sourcing::Result<()> {
        match event {
            CounterEvent::Incremented => self.count += 1,
            CounterEvent::Decremented => self.count -= 1,
            CounterEvent::Reset => self.count = 0,
        }
        Ok(())
    }
}

fn main() -> time_rs_sourcing::Result<()> {
    // Create a temporary directory for the example
    let temp_dir = std::env::temp_dir().join("time_rs_sourcing_example");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    println!("Event Sourcing Example");
    println!("======================\n");

    // Initialize an event store
    println!("Initializing event store at: {}", temp_dir.display());
    let mut store = GixEventStore::init(&temp_dir)?;

    // Create and store events
    println!("\nStoring events:");
    let events = vec![
        CounterEvent::Incremented,
        CounterEvent::Incremented,
        CounterEvent::Decremented,
        CounterEvent::Incremented,
        CounterEvent::Reset,
        CounterEvent::Incremented,
        CounterEvent::Incremented,
    ];

    for event in &events {
        let commit_id = store.append(event)?;
        println!("  {:?} -> commit {}", event, &commit_id[..8]);
    }

    // Read all events back
    println!("\nReading events from store:");
    let stored_events = store.read_all::<CounterEvent>()?;
    println!("  Found {} events", stored_events.len());

    // Replay events to rebuild state
    println!("\nReplaying events:");
    let counter = Counter::replay(stored_events.iter().map(|s| s.event.clone()))?;
    println!("  Final counter value: {}", counter.count);

    // Show event details
    println!("\nEvent details:");
    for (i, stored) in stored_events.iter().enumerate() {
        println!(
            "  Event {}: {:?}",
            i + 1,
            stored.event
        );
        println!("    Commit: {}", &stored.metadata.commit_id[..8]);
        println!("    File: {}", stored.metadata.file_path);
        println!("    Author: {}", stored.metadata.author);
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✓ Example completed successfully!");

    Ok(())
}
