// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Git-based event store implementation using gix

use std::path::{Path, PathBuf};
use std::process::Command;

use gix::bstr::ByteSlice;

use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventMetadata, StoredEvent};
use crate::store::EventStore;

/// A git-based event store that uses gix for git operations
///
/// Each event is stored as a JSON file in the repository with exactly one file per commit.
///
/// # Example
///
/// <!-- This example uses no_run because it requires filesystem operations -->
/// ```no_run
/// use time_rs_sourcing::{GixEventStore, EventStore};
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
/// let mut store = GixEventStore::init("./my-events")?;
///
/// // Append an event
/// let my_event = MyEvent::Created;
/// let commit_id = store.append(&my_event)?;
/// println!("Event stored in commit: {}", commit_id);
///
/// // Read all events
/// let events = store.read_all::<MyEvent>()?;
/// # Ok(())
/// # }
/// ```
pub struct GixEventStore {
    repo: gix::Repository,
    work_dir: PathBuf,
    events_dir: PathBuf,
}

impl GixEventStore {
    /// Create a new event store from an existing git repository
    ///
    /// # Example
    ///
    /// <!-- This example uses no_run because it requires filesystem operations -->
    /// ```no_run
    /// use time_rs_sourcing::GixEventStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = GixEventStore::open("./existing-repo")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be opened
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let work_dir = path.as_ref().to_path_buf();
        let repo = gix::open(&work_dir)
            .map_err(|e| EventSourcingError::RepositoryError(e.to_string()))?;

        let events_dir = work_dir.join("events");
        std::fs::create_dir_all(&events_dir)?;

        Ok(Self {
            repo,
            work_dir,
            events_dir,
        })
    }

    /// Initialize a new git repository for event storage
    ///
    /// # Example
    ///
    /// <!-- This example uses no_run because it requires filesystem operations -->
    /// ```no_run
    /// use time_rs_sourcing::GixEventStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = GixEventStore::init("./new-events")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be initialized
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let work_dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&work_dir)?;

        let repo = gix::init(&work_dir)
            .map_err(|e| EventSourcingError::RepositoryError(e.to_string()))?;

        let events_dir = work_dir.join("events");
        std::fs::create_dir_all(&events_dir)?;

        Ok(Self {
            repo,
            work_dir,
            events_dir,
        })
    }

    fn generate_event_filename(event_type: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_millis();
        format!("{}-{}.json", timestamp, event_type.replace("::", "_"))
    }

    fn git_add(&self, file_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("add")
            .arg(file_path)
            .current_dir(&self.work_dir)
            .output()?;

        if !output.status.success() {
            return Err(EventSourcingError::GitError(format!(
                "git add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn git_commit(&self, message: &str) -> Result<String> {
        // Configure git user for this repository if not already configured
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Event Store")
            .current_dir(&self.work_dir)
            .output()
            .ok();

        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("events@time-rs.dev")
            .current_dir(&self.work_dir)
            .output()
            .ok();

        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&self.work_dir)
            .output()?;

        if !output.status.success() {
            return Err(EventSourcingError::GitError(format!(
                "git commit failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Get the commit hash
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&self.work_dir)
            .output()?;

        if !output.status.success() {
            return Err(EventSourcingError::GitError(
                "Failed to get commit hash".to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl EventStore for GixEventStore {
    fn append<E: Event>(&mut self, event: &E) -> Result<String> {
        // Serialize the event to JSON
        let json = serde_json::to_string_pretty(event)?;

        // Generate a unique filename for this event
        let filename = Self::generate_event_filename(event.name());
        let event_path = self.events_dir.join(&filename);
        let relative_path = PathBuf::from("events").join(&filename);

        // Write the event to a file
        std::fs::write(&event_path, json)?;

        // Add and commit the file
        self.git_add(&relative_path)?;
        let commit_message = format!("Add event: {}", event.name());
        let commit_id = self.git_commit(&commit_message)?;

        Ok(commit_id)
    }

    fn read_all<E: Event>(&self) -> Result<Vec<StoredEvent<E>>> {
        let mut events = Vec::new();

        // Get the HEAD commit
        let mut head = self
            .repo
            .head()
            .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

        let head_commit = head
            .peel_to_commit()
            .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

        // Walk through all commits in reverse chronological order
        let mut commit_events = Vec::new();

        for commit_result in head_commit.ancestors().all().ok().into_iter().flatten() {
            let commit_info = commit_result
                .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

            let commit = commit_info
                .object()
                .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

            // Get the message to see if this is an event commit
            let message = commit.message().ok().map(|m| m.title.to_string());
            if let Some(msg) = message
                && !msg.starts_with("Add event:")
            {
                // Skip non-event commits
                continue;
            }

            let tree = commit
                .tree()
                .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

            // Find files in the events directory
            let events_path = gix::path::from_bstr(b"events".as_bstr());
            if let Ok(entry) = tree.lookup_entry_by_path(&events_path)
                && let Some(entry) = entry
                && entry.mode().is_tree()
            {
                let events_tree = entry
                    .object()
                    .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

                let subtree = events_tree
                    .try_into_tree()
                    .map_err(|_| EventSourcingError::GitError("Not a tree".to_string()))?;

                // For each commit, we should only have one event file (the new one)
                // To find which file is new in this commit, we need to compare with parent
                // For simplicity, we'll just take the most recent file by filename
                // Note: This assumes filenames are timestamped and monotonically increasing.
                // In practice, the git commit order is the authoritative event order.
                let mut max_file: Option<(String, E, EventMetadata)> = None;

                for entry in subtree.iter() {
                    let entry = entry
                        .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

                    if entry.mode().is_blob() {
                        let filename = entry.filename().to_string();
                        let blob = entry
                            .object()
                            .map_err(|e| EventSourcingError::GitError(e.to_string()))?;

                        let blob_obj = blob.try_into_blob().map_err(|_| {
                            EventSourcingError::GitError("Not a blob".to_string())
                        })?;

                        let content = blob_obj.data.to_str().map_err(|_| {
                            EventSourcingError::GitError("Invalid UTF-8 in file".to_string())
                        })?;

                        if let Ok(event) = serde_json::from_str::<E>(content) {
                            // Get filename as UTF-8 string, skip if invalid
                            let Ok(filename_str) = entry.filename().to_str() else {
                                continue;
                            };
                            
                            let file_path = format!("events/{filename_str}");


                            let metadata = EventMetadata {
                                commit_id: commit.id.to_string(),
                                timestamp: commit.time().ok().map_or(0, |t| t.seconds),
                                author: commit
                                    .author()
                                    .ok()
                                    .map_or_else(|| "unknown".to_string(), |a| a.name.to_string()),
                                file_path,
                            };

                            // Keep track of the file with the largest filename (most recent by timestamp)
                            if max_file.as_ref().is_none_or(|(f, _, _)| filename > *f) {
                                max_file = Some((filename, event, metadata));
                            }
                        }
                    }
                }

                // Add only the most recent event from this commit
                if let Some((_, event, metadata)) = max_file {
                    commit_events.push(StoredEvent { event, metadata });
                }
            }
        }

        // Reverse to get chronological order (oldest first)
        commit_events.reverse();
        events.extend(commit_events);

        Ok(events)
    }

    fn read_from<E: Event>(&self, _commit_id: &str) -> Result<Vec<StoredEvent<E>>> {
        // TODO: Implement efficient reading from a specific commit
        // For now, reading all events is the implemented behavior
        // A more efficient implementation would start from the given commit
        // and only read newer events
        self.read_all()
    }

    fn latest_commit(&self) -> Result<Option<String>> {
        let head = self.repo.head().ok();
        let commit = head.and_then(|mut h| h.peel_to_commit().ok());
        Ok(commit.map(|c| c.id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use time_rs_derive::Message;

    #[derive(Debug, Clone, PartialEq, Eq, Message, Serialize, Deserialize)]
    struct TestEvent {
        data: String,
    }

    #[rstest]
    fn can_create_event_store() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let store = GixEventStore::init(temp_dir.path());
        assert!(store.is_ok());
    }

    #[rstest]
    fn can_append_event() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();

        let event = TestEvent {
            data: "test event".to_string(),
        };

        let result = store.append(&event);
        assert!(result.is_ok());
    }

    #[rstest]
    fn can_read_appended_events() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();

        let event1 = TestEvent {
            data: "first event".to_string(),
        };
        let event2 = TestEvent {
            data: "second event".to_string(),
        };

        store.append(&event1).unwrap();
        store.append(&event2).unwrap();

        let events: Vec<StoredEvent<TestEvent>> = store.read_all().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event, event1);
        assert_eq!(events[1].event, event2);
    }

    #[rstest]
    fn events_are_in_chronological_order() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();

        for i in 0..5 {
            let event = TestEvent {
                data: format!("event {i}"),
            };
            store.append(&event).unwrap();
        }

        let events: Vec<StoredEvent<TestEvent>> = store.read_all().unwrap();
        assert_eq!(events.len(), 5);

        for (i, stored) in events.iter().enumerate() {
            assert_eq!(stored.event.data, format!("event {i}"));
        }
    }

    #[rstest]
    fn each_event_has_metadata() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();

        let event = TestEvent {
            data: "test".to_string(),
        };
        let commit_id = store.append(&event).unwrap();

        let events: Vec<StoredEvent<TestEvent>> = store.read_all().unwrap();
        assert_eq!(events.len(), 1);

        let metadata = &events[0].metadata;
        assert_eq!(metadata.commit_id, commit_id);
        assert!(metadata.timestamp > 0);
        assert!(!metadata.author.is_empty());
        assert!(metadata.file_path.starts_with("events/"));
    }

    #[rstest]
    fn one_file_per_commit() {
        use std::process::Command;
        
        const GIT_COMMIT_HASH_LENGTH: usize = 40;
        
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let mut store = GixEventStore::init(temp_dir.path()).unwrap();

        // Add multiple events
        for i in 0..3 {
            let event = TestEvent {
                data: format!("event {i}"),
            };
            store.append(&event).unwrap();
        }

        // Check each commit has exactly one file
        let output = Command::new("git")
            .arg("log")
            .arg("--pretty=format:%H")
            .arg("--name-only")
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        let log = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = log.lines().collect();

        // Count files per commit (format is: commit_hash, blank line, file1, file2, ..., blank line, next commit)
        let mut i = 0;
        let mut commit_count = 0;
        while i < lines.len() {
            if lines[i].len() == GIT_COMMIT_HASH_LENGTH {
                // This is a commit hash
                commit_count += 1;
                i += 1;
                
                if i < lines.len() && lines[i].is_empty() {
                    i += 1; // Skip blank line
                }
                
                // Count files for this commit
                let mut file_count = 0;
                while i < lines.len() && !lines[i].is_empty() && lines[i].len() != GIT_COMMIT_HASH_LENGTH {
                    if lines[i].starts_with("events/") {
                        file_count += 1;
                    }
                    i += 1;
                }
                
                // Each event commit should have exactly one file
                if file_count > 0 {
                    assert_eq!(file_count, 1, "Commit should have exactly one file");
                }
            } else {
                i += 1;
            }
        }
        
        assert!(commit_count >= 3, "Should have at least 3 event commits");
    }
}
