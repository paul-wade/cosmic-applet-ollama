// SPDX-License-Identifier: GPL-3.0

//! COSMIC Ollama Applet - A panel applet for chatting with local AI models.
//!
//! This applet provides quick access to Ollama-powered AI assistance from
//! the COSMIC desktop panel. It automatically gathers system context like
//! clipboard content, selected text, and recent errors to provide relevant help.

use crate::config::Config;
use crate::context::Context;
use crate::ollama::{self, Client as OllamaClient};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Length, Limits, Subscription, window::Id};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::{theme, widget};

/// Application identifier for COSMIC/freedesktop.
pub const APP_ID: &str = "com.github.paulwade.cosmic-applet-ollama";

/// The main application state.
#[derive(Default)]
pub struct AppModel {
    /// COSMIC application core.
    core: cosmic::Core,
    /// Active popup window ID.
    popup: Option<Id>,
    /// Application configuration.
    config: Config,
    /// Current text input value.
    input_text: String,
    /// Chat message history as (role, content) pairs.
    messages: Vec<(String, String)>,
    /// Whether we're waiting for an AI response.
    waiting: bool,
}

/// Application messages for state updates.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the popup window visibility.
    TogglePopup,
    /// Handle popup window close.
    PopupClosed(Id),
    /// Configuration update from cosmic-config.
    UpdateConfig(Config),
    /// Text input changed.
    InputChanged(String),
    /// User submitted a message.
    Submit,
    /// Received response from Ollama.
    OllamaResult(Result<String, String>),
    /// Clear chat history.
    ClearChat,
}

/// Send a message to Ollama with system context and optional web search.
async fn send_to_ollama(
    url: String,
    model: String,
    messages: Vec<(String, String)>,
    query: String,
) -> Result<String, String> {
    // Gather context with web search if query suggests it
    let context = Context::gather_with_search(&query).await;
    let system_prompt = context.format(ollama::DEFAULT_SYSTEM_PROMPT);
    OllamaClient::new(url, model)
        .chat(system_prompt, messages)
        .await
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

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
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|ctx| match Config::get_entry(&ctx) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        let app = AppModel {
            core,
            config,
            messages: vec![(
                "assistant".to_string(),
                "Hi! I'm your local AI assistant. Copy text for context, then ask me anything."
                    .to_string(),
            )],
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
        let header = self.build_header();
        let chat_content = self.build_chat_content();
        let input_row = self.build_input_row();

        let content = widget::column()
            .spacing(theme::active().cosmic().spacing.space_xs)
            .push(header)
            .push(widget::divider::horizontal::light())
            .push(chat_content)
            .push(widget::divider::horizontal::light())
            .push(input_row)
            .padding(theme::active().cosmic().spacing.space_s);

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
                return self.handle_submit();
            }
            Message::OllamaResult(result) => {
                self.handle_ollama_result(result);
            }
            Message::TogglePopup => {
                return self.handle_toggle_popup();
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ClearChat => {
                self.messages.clear();
                self.messages.push((
                    "assistant".to_string(),
                    "Chat cleared. How can I help?".to_string(),
                ));
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

// Private helper methods
impl AppModel {
    fn build_header(&self) -> Element<'_, Message> {
        let model_label = widget::text::body(&self.config.model).width(Length::Fill);

        let clear_btn = widget::button::icon(widget::icon::from_name("edit-clear-symbolic"))
            .padding(theme::active().cosmic().spacing.space_xxs)
            .on_press(Message::ClearChat);

        widget::row()
            .align_y(Alignment::Center)
            .spacing(theme::active().cosmic().spacing.space_xs)
            .push(model_label)
            .push(clear_btn)
            .into()
    }

    fn build_chat_content(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;
        let mut chat_column = widget::column().spacing(spacing.space_xs);

        for (role, content) in &self.messages {
            let message_widget = self.build_message_bubble(role, content);
            chat_column = chat_column.push(message_widget);
        }

        if self.waiting {
            let thinking = widget::container(widget::text::body("Thinking...").width(Length::Fill))
                .class(theme::Container::Card)
                .padding(spacing.space_s);
            chat_column = chat_column.push(thinking);
        }

        widget::scrollable(chat_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn build_message_bubble<'a>(&'a self, role: &str, content: &'a str) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let (prefix, container_class) = if role == "user" {
            ("You", theme::Container::Primary)
        } else {
            ("AI", theme::Container::Card)
        };

        let text_content = widget::text(content).width(Length::Fill);
        let label = widget::text::caption(prefix);

        let bubble_content = widget::column()
            .spacing(spacing.space_xxs)
            .push(label)
            .push(text_content);

        widget::container(bubble_content)
            .class(container_class)
            .padding(spacing.space_s)
            .width(Length::Fill)
            .into()
    }

    fn build_input_row(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let input = widget::text_input("Type a message...", &self.input_text)
            .on_input(Message::InputChanged)
            .on_submit(|_| Message::Submit)
            .width(Length::Fill);

        let send_btn = if self.waiting {
            widget::button::icon(widget::icon::from_name("process-working-symbolic"))
                .padding(spacing.space_xxs)
        } else {
            widget::button::icon(widget::icon::from_name("go-next-symbolic"))
                .padding(spacing.space_xxs)
                .on_press(Message::Submit)
        };

        widget::row()
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center)
            .push(input)
            .push(send_btn)
            .into()
    }

    fn handle_submit(&mut self) -> Task<cosmic::Action<Message>> {
        if self.input_text.trim().is_empty() || self.waiting {
            return Task::none();
        }

        let query = self.input_text.clone();
        self.messages.push(("user".to_string(), query.clone()));
        self.input_text.clear();
        self.waiting = true;

        let url = self.config.ollama_url.clone();
        let model = self.config.model.clone();
        let messages = self.messages.clone();

        Task::perform(
            async move { send_to_ollama(url, model, messages, query).await },
            |result| cosmic::Action::App(Message::OllamaResult(result)),
        )
    }

    fn handle_ollama_result(&mut self, result: Result<String, String>) {
        self.waiting = false;
        match result {
            Ok(content) => {
                self.messages.push(("assistant".to_string(), content));
            }
            Err(err) => {
                self.messages
                    .push(("assistant".to_string(), format!("Error: {}", err)));
            }
        }
    }

    fn handle_toggle_popup(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(p) = self.popup.take() {
            return destroy_popup(p);
        }

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
