// SPDX-License-Identifier: GPL-3.0

//! Ollama API client for chat completions.
//!
//! Handles communication with the local Ollama server.

use crate::config;
use serde::{Deserialize, Serialize};

/// Default system prompt for the assistant.
pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You are a helpful Pop!_OS and Linux assistant. You have deep knowledge of \
Pop!_OS, System76 hardware, COSMIC desktop, Ubuntu/Debian package management, \
systemd, the Linux kernel, and general Linux troubleshooting. Keep responses \
concise as they display in a small panel. When relevant, provide specific \
commands the user can run.";

/// A message in the Ollama chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Request payload for Ollama chat API.
#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

/// Response from Ollama chat API.
#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    message: Message,
}

/// Ollama client for making API requests.
pub struct Client {
    url: String,
    model: String,
    http: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self::new(config::DEFAULT_OLLAMA_URL, config::DEFAULT_MODEL)
    }
}

impl Client {
    /// Create a new Ollama client with custom URL and model.
    pub fn new(url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            model: model.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Send a chat completion request to Ollama.
    ///
    /// # Arguments
    /// * `system_prompt` - The system message providing context and instructions
    /// * `messages` - The conversation history as (role, content) tuples
    ///
    /// # Returns
    /// The assistant's response content, or an error message.
    pub async fn chat(
        &self,
        system_prompt: String,
        messages: Vec<(String, String)>,
    ) -> Result<String, String> {
        let mut ollama_messages = vec![Message::system(system_prompt)];

        ollama_messages.extend(
            messages
                .into_iter()
                .map(|(role, content)| Message { role, content }),
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
        };

        let response = self
            .http
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(chat_response.message.content)
    }
}
