// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Command pattern for event sourcing
//!
//! Commands represent intentions to change state. They are validated and processed
//! to generate events that describe the actual state changes.

use crate::error::Result;
use crate::event::Event;
use crate::message::Message;

/// A command represents an intention to change the state of an aggregate
///
/// Commands are validated before being processed, and successful processing
/// results in one or more events being generated.
///
/// # Example
///
/// ```
/// use time_rs_sourcing::{Command, Result};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct CreateAccount {
///     account_id: String,
///     initial_balance: i64,
/// }
///
/// impl time_rs_sourcing::message::Message for CreateAccount {
///     fn name(&self) -> &'static str {
///         "CreateAccount"
///     }
/// }
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// enum AccountEvent {
///     AccountCreated { account_id: String, balance: i64 },
/// }
///
/// impl time_rs_sourcing::message::Message for AccountEvent {
///     fn name(&self) -> &'static str {
///         "AccountEvent"
///     }
/// }
///
/// impl Command for CreateAccount {
///     type Event = AccountEvent;
///
///     fn execute(&self) -> Result<Vec<Self::Event>> {
///         // Validate command
///         if self.initial_balance < 0 {
///             return Err(time_rs_sourcing::EventSourcingError::RepositoryError(
///                 "Initial balance cannot be negative".to_string()
///             ));
///         }
///
///         // Generate events
///         Ok(vec![AccountEvent::AccountCreated {
///             account_id: self.account_id.clone(),
///             balance: self.initial_balance,
///         }])
///     }
/// }
/// ```
pub trait Command: Message {
    /// The type of events this command can generate
    type Event: Event;

    /// Execute the command, validating it and generating events
    ///
    /// # Errors
    ///
    /// Returns an error if the command validation fails
    fn execute(&self) -> Result<Vec<Self::Event>>;
}

/// A command handler processes commands and persists the resulting events
///
/// # Example
///
/// ```
/// use time_rs_sourcing::{Command, CommandHandler, EventStore};
///
/// struct MyCommandHandler<S> {
///     store: S,
/// }
///
/// impl<S: EventStore> CommandHandler<S> for MyCommandHandler<S> {
///     fn handle<C: Command>(&mut self, command: &C) -> time_rs_sourcing::Result<Vec<String>> {
///         let events = command.execute()?;
///         let mut commit_ids = Vec::new();
///
///         for event in events {
///             let commit_id = self.store.append(&event)?;
///             commit_ids.push(commit_id);
///         }
///
///         Ok(commit_ids)
///     }
/// }
/// ```
pub trait CommandHandler<S> {
    /// Handle a command by executing it and storing the resulting events
    ///
    /// # Errors
    ///
    /// Returns an error if command execution or event storage fails
    fn handle<C: Command>(&mut self, command: &C) -> Result<Vec<String>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestCommand {
        value: i32,
    }

    impl Message for TestCommand {
        fn name(&self) -> &'static str {
            "TestCommand"
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl Message for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    impl Command for TestCommand {
        type Event = TestEvent;

        fn execute(&self) -> Result<Vec<Self::Event>> {
            if self.value < 0 {
                return Err(crate::error::EventSourcingError::RepositoryError(
                    "Value cannot be negative".to_string(),
                ));
            }
            Ok(vec![TestEvent { value: self.value }])
        }
    }

    #[test]
    fn test_command_execution_success() {
        let command = TestCommand { value: 42 };
        let events = command.execute().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].value, 42);
    }

    #[test]
    fn test_command_execution_failure() {
        let command = TestCommand { value: -1 };
        let result = command.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_command_generates_multiple_events() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct MultiEventCommand;

        impl Message for MultiEventCommand {
            fn name(&self) -> &'static str {
                "MultiEventCommand"
            }
        }

        impl Command for MultiEventCommand {
            type Event = TestEvent;

            fn execute(&self) -> Result<Vec<Self::Event>> {
                Ok(vec![
                    TestEvent { value: 1 },
                    TestEvent { value: 2 },
                    TestEvent { value: 3 },
                ])
            }
        }

        let command = MultiEventCommand;
        let events = command.execute().unwrap();
        assert_eq!(events.len(), 3);
    }
}
