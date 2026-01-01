// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Integration tests for the event sourcing system

use serde::{Deserialize, Serialize};
use time_rs_sourcing::{Aggregate, EventStore, GixEventStore};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum BankAccountEvent {
    AccountOpened { balance: i64 },
    MoneyDeposited { amount: i64 },
    MoneyWithdrawn { amount: i64 },
    AccountClosed,
}

impl time_rs_sourcing::message::Message for BankAccountEvent {
    fn name(&self) -> &'static str {
        "BankAccountEvent"
    }
}

#[derive(Debug, Default)]
struct BankAccount {
    balance: i64,
    is_open: bool,
}

impl Aggregate for BankAccount {
    type Event = BankAccountEvent;

    fn apply(&mut self, event: &Self::Event) -> time_rs_sourcing::Result<()> {
        match event {
            BankAccountEvent::AccountOpened { balance } => {
                self.balance = *balance;
                self.is_open = true;
            }
            BankAccountEvent::MoneyDeposited { amount } => {
                self.balance += amount;
            }
            BankAccountEvent::MoneyWithdrawn { amount } => {
                self.balance -= amount;
            }
            BankAccountEvent::AccountClosed => {
                self.is_open = false;
            }
        }
        Ok(())
    }
}

#[test]
fn test_end_to_end_event_sourcing() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let mut store = GixEventStore::init(temp_dir.path()).unwrap();

    // Create a series of events
    let events = vec![
        BankAccountEvent::AccountOpened { balance: 0 },
        BankAccountEvent::MoneyDeposited { amount: 100 },
        BankAccountEvent::MoneyDeposited { amount: 50 },
        BankAccountEvent::MoneyWithdrawn { amount: 30 },
        BankAccountEvent::MoneyDeposited { amount: 20 },
    ];

    // Store each event
    let mut commit_ids = Vec::new();
    for event in &events {
        let commit_id = store.append(event).unwrap();
        commit_ids.push(commit_id);
    }

    // Verify all commits are unique
    for i in 0..commit_ids.len() {
        for j in (i + 1)..commit_ids.len() {
            assert_ne!(commit_ids[i], commit_ids[j]);
        }
    }

    // Read all events back
    let stored_events = store.read_all::<BankAccountEvent>().unwrap();
    assert_eq!(stored_events.len(), events.len());

    // Verify events match
    for (i, stored) in stored_events.iter().enumerate() {
        assert_eq!(stored.event, events[i]);
        assert!(!stored.metadata.commit_id.is_empty());
        assert!(stored.metadata.timestamp > 0);
    }

    // Replay events to rebuild state
    let account = BankAccount::replay(stored_events.iter().map(|s| s.event.clone())).unwrap();
    assert_eq!(account.balance, 140); // 0 + 100 + 50 - 30 + 20
    assert!(account.is_open);
}

#[test]
fn test_multiple_stores_same_repository() {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // First store: initialize and add events
    {
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();
        store
            .append(&BankAccountEvent::AccountOpened { balance: 0 })
            .unwrap();
        store
            .append(&BankAccountEvent::MoneyDeposited { amount: 100 })
            .unwrap();
    }

    // Second store: open existing and read events
    {
        let store = GixEventStore::open(temp_dir.path()).unwrap();
        let events = store.read_all::<BankAccountEvent>().unwrap();
        assert_eq!(events.len(), 2);

        let account = BankAccount::replay(events.iter().map(|s| s.event.clone())).unwrap();
        assert_eq!(account.balance, 100);
    }

    // Third store: add more events
    {
        let mut store = GixEventStore::open(temp_dir.path()).unwrap();
        store
            .append(&BankAccountEvent::MoneyWithdrawn { amount: 50 })
            .unwrap();

        let events = store.read_all::<BankAccountEvent>().unwrap();
        assert_eq!(events.len(), 3);

        let account = BankAccount::replay(events.iter().map(|s| s.event.clone())).unwrap();
        assert_eq!(account.balance, 50);
    }
}

#[test]
fn test_latest_commit() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let mut store = GixEventStore::init(temp_dir.path()).unwrap();

    // No events yet, should have no commits
    let latest = store.latest_commit().unwrap();
    assert!(latest.is_none());

    // Add an event
    let commit_id = store
        .append(&BankAccountEvent::AccountOpened { balance: 0 })
        .unwrap();

    // Should have a commit now
    let latest = store.latest_commit().unwrap();
    assert_eq!(latest, Some(commit_id));

    // Add another event
    let second_commit = store
        .append(&BankAccountEvent::MoneyDeposited { amount: 100 })
        .unwrap();

    // Latest should be the second commit
    let latest = store.latest_commit().unwrap();
    assert_eq!(latest, Some(second_commit));
}

#[test]
fn test_event_chronological_order() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let mut store = GixEventStore::init(temp_dir.path()).unwrap();

    // Add events with clear ordering
    let events = vec![
        BankAccountEvent::AccountOpened { balance: 0 },
        BankAccountEvent::MoneyDeposited { amount: 10 },
        BankAccountEvent::MoneyDeposited { amount: 20 },
        BankAccountEvent::MoneyDeposited { amount: 30 },
    ];

    for event in &events {
        store.append(event).unwrap();
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Read events and verify order
    let stored = store.read_all::<BankAccountEvent>().unwrap();
    assert_eq!(stored.len(), events.len());

    // Timestamps should be in increasing order
    for i in 1..stored.len() {
        assert!(
            stored[i].metadata.timestamp >= stored[i - 1].metadata.timestamp,
            "Events not in chronological order"
        );
    }

    // Events should match the order they were added
    for (i, stored_event) in stored.iter().enumerate() {
        assert_eq!(stored_event.event, events[i]);
    }
}

#[test]
fn test_complex_aggregate_state() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let mut store = GixEventStore::init(temp_dir.path()).unwrap();

    // Simulate a full account lifecycle
    let events = vec![
        BankAccountEvent::AccountOpened { balance: 1000 },
        BankAccountEvent::MoneyDeposited { amount: 500 },
        BankAccountEvent::MoneyWithdrawn { amount: 200 },
        BankAccountEvent::MoneyDeposited { amount: 300 },
        BankAccountEvent::MoneyWithdrawn { amount: 100 },
        BankAccountEvent::AccountClosed,
    ];

    for event in &events {
        store.append(event).unwrap();
    }

    let stored = store.read_all::<BankAccountEvent>().unwrap();
    let account = BankAccount::replay(stored.iter().map(|s| s.event.clone())).unwrap();

    assert_eq!(account.balance, 1500); // 1000 + 500 - 200 + 300 - 100
    assert!(!account.is_open);
}
