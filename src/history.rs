// SPDX-License-Identifier: GPL-3.0

//! Chat history persistence module.
//!
//! Saves and loads chat history to/from the XDG data directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

/// Maximum number of messages to keep in history.
pub const MAX_HISTORY_SIZE: usize = 100;

/// A single chat message in the history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
}

/// Chat history container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatHistory {
    /// Version for future schema migrations.
    pub version: u32,
    /// List of chat messages.
    pub messages: Vec<HistoryMessage>,
}

impl ChatHistory {
    /// Current history format version.
    const CURRENT_VERSION: u32 = 1;

    /// Create a new empty history.
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            messages: Vec::new(),
        }
    }

    /// Create history from existing messages.
    pub fn from_messages(messages: Vec<(String, String)>) -> Self {
        let history_messages: Vec<HistoryMessage> = messages
            .into_iter()
            .map(|(role, content)| HistoryMessage { role, content })
            .collect();

        Self {
            version: Self::CURRENT_VERSION,
            messages: history_messages,
        }
    }

    /// Convert history to message tuples for the app.
    pub fn to_messages(&self) -> Vec<(String, String)> {
        self.messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect()
    }

    /// Trim history to max size, keeping most recent messages.
    pub fn trim_to_limit(&mut self) {
        if self.messages.len() > MAX_HISTORY_SIZE {
            let excess = self.messages.len() - MAX_HISTORY_SIZE;
            self.messages.drain(0..excess);
        }
    }
}

/// Get the path to the history file.
fn history_file_path() -> Option<PathBuf> {
    // Use XDG_DATA_HOME or ~/.local/share
    let data_dir = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local/share")
        });

    let app_dir = data_dir.join("cosmic-applet-ollama");
    Some(app_dir.join("history.json"))
}

/// Load chat history from disk.
pub fn load_history() -> ChatHistory {
    let Some(path) = history_file_path() else {
        return ChatHistory::new();
    };

    if !path.exists() {
        return ChatHistory::new();
    }

    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return ChatHistory::new(),
    };

    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(history) => history,
        Err(_) => ChatHistory::new(),
    }
}

/// Save chat history to disk.
pub fn save_history(messages: &[(String, String)]) -> io::Result<()> {
    let Some(path) = history_file_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine history path",
        ));
    };

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut history = ChatHistory::from_messages(messages.to_vec());
    history.trim_to_limit();

    let file = fs::File::create(&path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &history)?;

    Ok(())
}

/// Clear saved history from disk.
pub fn clear_history() -> io::Result<()> {
    let Some(path) = history_file_path() else {
        return Ok(());
    };

    if path.exists() {
        fs::remove_file(&path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_roundtrip() {
        let messages = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];

        let history = ChatHistory::from_messages(messages.clone());
        let restored = history.to_messages();

        assert_eq!(restored, messages);
    }

    #[test]
    fn test_trim_to_limit() {
        let mut history = ChatHistory::new();
        for i in 0..(MAX_HISTORY_SIZE + 50) {
            history.messages.push(HistoryMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
            });
        }

        history.trim_to_limit();
        assert_eq!(history.messages.len(), MAX_HISTORY_SIZE);

        // Should keep the most recent messages
        assert_eq!(
            history.messages.last().unwrap().content,
            format!("Message {}", MAX_HISTORY_SIZE + 49)
        );
    }
}
