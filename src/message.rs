//! This module provides the messages use to send status updates back to the
//! caller of nixtract. This is, for instance, usefull when writing a UI for
//! nixtract.

use std::fmt;

pub enum Message {
    /// Started the describing of the given attribute path
    Started(usize, String),
    /// Completed the describing of the given attribute path
    Completed(usize, String),
    /// The derivation has already been described by another thread
    Skipped(usize, String),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Started(id, path) => write!(f, "Thread {} started {}", id, path),
            Message::Completed(id, path) => write!(f, "Thread {} completed {}", id, path),
            Message::Skipped(id, path) => write!(f, "Thread {} skipped {}", id, path),
        }
    }
}
