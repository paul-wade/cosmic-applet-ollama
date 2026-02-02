// SPDX-License-Identifier: GPL-3.0

//! System context gathering for enhanced AI responses.
//!
//! This module collects contextual information from the user's environment
//! to provide the AI with relevant background for better assistance.

use std::process::Command;

/// Maximum size for clipboard/selection content to avoid overwhelming the model.
const MAX_CONTENT_SIZE: usize = 2000;

/// Maximum size for journal error output.
const MAX_ERROR_SIZE: usize = 1500;

/// Collected system context for AI prompts.
#[derive(Default, Clone, Debug)]
pub struct Context {
    /// Clipboard content (Ctrl+C)
    pub clipboard: Option<String>,
    /// Primary selection (highlighted text, no copy needed)
    pub selection: Option<String>,
    /// System information (OS, kernel, memory)
    pub system_info: Option<String>,
    /// Recent system errors from journalctl
    pub recent_errors: Option<String>,
}

impl Context {
    /// Gather all available context from the system.
    pub fn gather() -> Self {
        let clipboard = Self::get_clipboard();
        let selection = Self::get_selection(&clipboard);

        Self {
            clipboard,
            selection,
            system_info: Self::get_system_info(),
            recent_errors: Self::get_recent_errors(),
        }
    }

    /// Build a formatted context string for the AI system prompt.
    pub fn format(&self, base_prompt: &str) -> String {
        let mut parts = vec![base_prompt.to_string()];

        if let Some(ref clip) = self.clipboard {
            parts.push(format!("\n\n## Clipboard:\n```\n{}\n```", clip));
        }
        if let Some(ref sel) = self.selection {
            parts.push(format!("\n\n## Selected text:\n```\n{}\n```", sel));
        }
        if let Some(ref info) = self.system_info {
            parts.push(format!("\n\n## System: {}", info));
        }
        if let Some(ref errs) = self.recent_errors {
            parts.push(format!("\n\n## Recent errors:\n```\n{}\n```", errs));
        }

        parts.join("")
    }

    fn get_clipboard() -> Option<String> {
        run_cmd("wl-paste", &["--no-newline"]).filter(|s| s.len() < MAX_CONTENT_SIZE)
    }

    fn get_selection(clipboard: &Option<String>) -> Option<String> {
        run_cmd("wl-paste", &["--primary", "--no-newline"])
            .filter(|s| s.len() < MAX_CONTENT_SIZE)
            // Exclude if same as clipboard (avoid duplicates)
            .filter(|s| clipboard.as_ref() != Some(s))
    }

    fn get_system_info() -> Option<String> {
        let distro = run_cmd("lsb_release", &["-d", "-s"]).unwrap_or_default();
        let kernel = run_cmd("uname", &["-r"]).unwrap_or_default();
        let mem = run_cmd("free", &["-h", "--si"]).and_then(|s| s.lines().nth(1).map(String::from));

        if !distro.is_empty() {
            Some(format!(
                "OS: {}, Kernel: {}, Memory: {}",
                distro,
                kernel,
                mem.unwrap_or_default()
            ))
        } else {
            None
        }
    }

    fn get_recent_errors() -> Option<String> {
        run_cmd("journalctl", &["-p", "err", "-n", "5", "--no-pager", "-q"])
            .filter(|s| s.len() < MAX_ERROR_SIZE)
    }
}

/// Execute a command and return trimmed stdout if successful.
fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
