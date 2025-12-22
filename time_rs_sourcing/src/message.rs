// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Deals with messages, which can be used as Events or Commands

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "derive")]
#[doc(inline)]
pub use time_rs_derive::Message;

/// A [`Message`] is a serializable and deserializable piece of data.
///
/// Any message does have a name, which is used to identify its type when deserializing.
pub trait Message {
    /// Returns the [`Message`]s type name
    fn name(&self) -> &'static str;
}

/// Arbitrary metadata that can be attached to a message
pub type Metadata = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An [`Envelope`] wraps the [`Message`]
pub struct Envelope<M>
where
    M: Message,
{
    /// The message itself
    pub message: M,
    /// Associated [`Metadata`]
    pub meta: Metadata,
}

impl<M> Envelope<M>
where
    M: Message,
{
    #[must_use]
    /// Sets the given `key` to the given `value` in the [`Self::meta`]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.meta.insert(key, value);
        self
    }
}

impl<M> From<M> for Envelope<M>
where
    M: Message,
{
    fn from(message: M) -> Self {
        Self {
            message,
            meta: Metadata::default(),
        }
    }
}

impl<M> PartialEq for Envelope<M>
where
    M: Message + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use rstest::rstest;
    use time_rs_derive::Message;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Message, Serialize)]
    struct StringMessage(pub &'static str);

    #[rstest]
    fn message_with_metadata_doesnt_affect_equality() {
        let message = Envelope {
            message: StringMessage("hello"),
            meta: Metadata::default(),
        };

        let other_message = message.clone().with_metadata("test".into(), "value".into());

        assert_eq!(message, other_message);
    }

    #[rstest]
    fn metadata_is_persistet() {
        let message = Envelope {
            message: StringMessage("bye"),
            meta: Metadata::default(),
        }
        .with_metadata("meta".into(), "data".into());

        assert_eq!("data", message.meta["meta"]);
    }
}
