// SPDX-License-Identifier: GPL-3.0

mod app;
mod config;
mod context;
mod i18n;
mod ollama;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    cosmic::applet::run::<app::AppModel>(())
}
