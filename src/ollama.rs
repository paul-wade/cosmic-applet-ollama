// SPDX-License-Identifier: GPL-3.0

//! Ollama API client for chat completions.
//!
//! Handles communication with the local Ollama server.

use crate::config;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Default system prompt for the assistant.
pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You are a helpful Linux assistant running locally. You help with Pop!_OS, System76, \
COSMIC desktop, Ubuntu/Debian, systemd, and general Linux.

CRITICAL - READ THIS FIRST:
- COSMIC desktop is BRAND NEW (2024+). Your training data is OUTDATED about it.
- Pop!_OS and COSMIC change frequently. DO NOT guess or assume features exist.
- If web search results are provided below, ALWAYS prioritize them over your training.
- The web results are CURRENT and ACCURATE - trust them over your built-in knowledge.

GUIDELINES:
- If no web results are provided and you're unsure, say \"I don't have current info on this.\"
- Never invent package names, commands, config paths, or features.
- When web results are provided, cite them: \"According to the search results...\"
- Keep responses concise - this displays in a small panel.
- Provide specific commands when relevant.
- For COSMIC questions without web results, suggest checking: https://system76.com/cosmic";

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
}

/// Request payload for Ollama chat API.
#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

/// Response from Ollama chat API (non-streaming).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    message: Message,
}

/// Streaming response chunk from Ollama.
#[derive(Debug, Clone, Deserialize)]
struct StreamChunk {
    message: Option<StreamMessage>,
    done: bool,
}

/// Response from Ollama tags API (model listing).
#[derive(Debug, Clone, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

/// Information about a single model from the tags API.
#[derive(Debug, Clone, Deserialize)]
struct ModelInfo {
    name: String,
    size: u64,
}

/// A model available in Ollama with display information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableModel {
    /// Model name/tag (e.g., "llama3.2:3b")
    pub name: String,
    /// Human-readable size (e.g., "2.0 GB")
    pub display_size: String,
}

/// Message content in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
struct StreamMessage {
    content: String,
}

/// Event sent during streaming response.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A chunk of content arrived.
    Chunk(String),
    /// Stream completed successfully.
    Done,
    /// An error occurred.
    Error(String),
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

/// Format bytes into human-readable size.
fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.0} MB", bytes as f64 / MB as f64)
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

    /// List available models from Ollama.
    ///
    /// Queries the /api/tags endpoint to get all installed models.
    pub async fn list_models(base_url: &str) -> Result<Vec<AvailableModel>, String> {
        // Convert chat URL to tags URL
        let tags_url = base_url
            .replace("/api/chat", "/api/tags")
            .replace("/api/generate", "/api/tags");

        let http = reqwest::Client::new();
        let response = http
            .get(&tags_url)
            .send()
            .await
            .map_err(|e| format!("Connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let tags_response: TagsResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(tags_response
            .models
            .into_iter()
            .map(|m| AvailableModel {
                name: m.name,
                display_size: format_size(m.size),
            })
            .collect())
    }

    /// Send a chat completion request to Ollama (non-streaming).
    ///
    /// # Arguments
    /// * `system_prompt` - The system message providing context and instructions
    /// * `messages` - The conversation history as (role, content) tuples
    ///
    /// # Returns
    /// The assistant's response content, or an error message.
    #[allow(dead_code)]
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

    /// Send a streaming chat request to Ollama.
    ///
    /// Returns a receiver that yields content chunks as they arrive.
    pub async fn chat_stream(
        &self,
        system_prompt: String,
        messages: Vec<(String, String)>,
    ) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel(32);

        let mut ollama_messages = vec![Message::system(system_prompt)];
        ollama_messages.extend(
            messages
                .into_iter()
                .map(|(role, content)| Message { role, content }),
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: true,
        };

        let http = self.http.clone();
        let url = self.url.clone();

        tokio::spawn(async move {
            let response = match http.post(&url).json(&request).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(StreamEvent::Error(format!("Connection error: {}", e)))
                        .await;
                    return;
                }
            };

            if !response.status().is_success() {
                let _ = tx
                    .send(StreamEvent::Error(format!(
                        "Ollama error: {}",
                        response.status()
                    )))
                    .await;
                return;
            }

            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        // Ollama returns newline-delimited JSON
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if line.is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<StreamChunk>(line) {
                                Ok(chunk) => {
                                    if chunk.done {
                                        let _ = tx.send(StreamEvent::Done).await;
                                        return;
                                    }
                                    if let Some(msg) = chunk.message
                                        && !msg.content.is_empty()
                                        && tx.send(StreamEvent::Chunk(msg.content)).await.is_err()
                                    {
                                        return; // Receiver dropped
                                    }
                                }
                                Err(e) => {
                                    let _ = tx
                                        .send(StreamEvent::Error(format!("Parse error: {}", e)))
                                        .await;
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(StreamEvent::Error(format!("Stream error: {}", e)))
                            .await;
                        return;
                    }
                }
            }

            // Stream ended without done flag
            let _ = tx.send(StreamEvent::Done).await;
        });

        rx
    }
}
