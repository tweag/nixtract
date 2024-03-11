//! This module provides the messages use to send status updates back to the
//! caller of nixtract. This is, for instance, usefull when writing a UI for
//! nixtract.

use std::fmt;

// Define an enum for the status
pub enum Status {
    Started,
    Completed,
    Skipped,
}

// Combine the struct and enum into a new Message struct
pub struct Message {
    pub status: Status,
    pub id: usize,
    pub path: String,
}

// Implement Display for the Status enum
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Started => write!(f, "started"),
            Status::Completed => write!(f, "completed"),
            Status::Skipped => write!(f, "skipped"),
        }
    }
}

// Implement Display for the Message enum
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Thread {} {} {}", self.id, self.status, self.path)
    }
}
