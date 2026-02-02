// SPDX-License-Identifier: GPL-3.0

//! Application configuration stored via cosmic-config.

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434/api/chat";
pub const DEFAULT_MODEL: &str = "llama3.2:3b";

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    /// Ollama API endpoint URL.
    pub ollama_url: String,
    /// Model to use for chat completions.
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_url: DEFAULT_OLLAMA_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }
}
