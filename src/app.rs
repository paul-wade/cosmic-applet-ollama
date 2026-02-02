// SPDX-License-Identifier: GPL-3.0

use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Length, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use serde::{Deserialize, Serialize};

const OLLAMA_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_MODEL: &str = "phi3:mini";
const SYSTEM_PROMPT: &str = "You are a helpful Pop!_OS and Linux assistant. You have deep knowledge of Pop!_OS, System76 hardware, COSMIC desktop, Ubuntu/Debian package management, systemd, the Linux kernel, and general Linux troubleshooting. Keep responses concise as they display in a small panel. When relevant, provide specific commands the user can run.";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

/// The application model stores app-specific state.
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    input_text: String,
    messages: Vec<(String, String)>,
    waiting: bool,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: cosmic::Core::default(),
            popup: None,
            config: Config::default(),
            input_text: String::new(),
            messages: Vec::new(),
            waiting: false,
        }
    }
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    InputChanged(String),
    Submit,
    OllamaResult(Result<String, String>),
}

fn get_clipboard() -> Option<String> {
    std::process::Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .filter(|s| !s.trim().is_empty() && s.len() < 2000)
}

async fn call_ollama(messages: Vec<(String, String)>, clipboard: Option<String>) -> Result<String, String> {
    let mut ollama_messages: Vec<OllamaMessage> = Vec::new();

    // Add system prompt with optional clipboard context
    let system_content = match clipboard {
        Some(clip) => format!("{}\n\nCurrent clipboard content for context:\n```\n{}\n```", SYSTEM_PROMPT, clip),
        None => SYSTEM_PROMPT.to_string(),
    };

    ollama_messages.push(OllamaMessage {
        role: "system".to_string(),
        content: system_content,
    });

    // Add conversation messages
    ollama_messages.extend(messages.iter().map(|(role, content)| OllamaMessage {
        role: role.clone(),
        content: content.clone(),
    }));

    let request = OllamaRequest {
        model: DEFAULT_MODEL.to_string(),
        messages: ollama_messages,
        stream: false,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(OLLAMA_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama error: {}", response.status()));
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(ollama_response.message.content)
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.paulwade.cosmic-applet-ollama";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            messages: vec![
                ("assistant".to_string(), "Hi! I'm your Pop!_OS assistant. Copy text to clipboard for context, then ask me anything.".to_string()),
            ],
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("user-available-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut chat_column = widget::column().spacing(8).padding(8);

        for (role, content) in &self.messages {
            let label = if role == "user" {
                format!("You: {}", content)
            } else {
                format!("AI: {}", content)
            };
            chat_column = chat_column.push(
                widget::text(label)
                    .width(Length::Fill)
            );
        }

        if self.waiting {
            chat_column = chat_column.push(
                widget::text("AI: thinking...")
                    .width(Length::Fill)
            );
        }

        let chat_scroll = widget::scrollable(chat_column)
            .height(Length::Fill)
            .width(Length::Fill);

        let input = widget::text_input("Type a message...", &self.input_text)
            .on_input(Message::InputChanged)
            .on_submit(|_| Message::Submit)
            .width(Length::Fill);

        let send_btn = if self.waiting {
            widget::button::standard("...")
        } else {
            widget::button::standard("Send")
                .on_press(Message::Submit)
        };

        let input_row = widget::row()
            .spacing(8)
            .push(input)
            .push(send_btn);

        let content = widget::column()
            .spacing(12)
            .padding(12)
            .push(chat_scroll)
            .push(input_row);

        self.core.applet.popup_container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::InputChanged(text) => {
                self.input_text = text;
            }
            Message::Submit => {
                if !self.input_text.trim().is_empty() && !self.waiting {
                    self.messages.push(("user".to_string(), self.input_text.clone()));
                    self.input_text.clear();
                    self.waiting = true;

                    let messages = self.messages.clone();
                    let clipboard = get_clipboard();
                    return Task::perform(
                        async move { call_ollama(messages, clipboard).await },
                        |result| cosmic::Action::App(Message::OllamaResult(result)),
                    );
                }
            }
            Message::OllamaResult(result) => {
                self.waiting = false;
                match result {
                    Ok(content) => {
                        self.messages.push(("assistant".to_string(), content));
                    }
                    Err(err) => {
                        self.messages.push(("assistant".to_string(), format!("Error: {}", err)));
                    }
                }
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(400.0)
                        .min_width(350.0)
                        .min_height(400.0)
                        .max_height(600.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
