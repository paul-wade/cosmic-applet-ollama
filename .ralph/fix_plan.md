# COSMIC Ollama Applet - Development Plan

## Completed Features
- [x] Basic chat UI with COSMIC styling
- [x] Message bubbles with theme colors
- [x] Configurable model via cosmic-config
- [x] Context gathering (clipboard, selection, system info, journal errors)
- [x] Web search integration (DuckDuckGo)
- [x] Honest system prompt about knowledge limits
- [x] CI/CD pipeline with GitHub Actions
- [x] Flatpak manifest

## Phase 1: Streaming Responses (Priority: HIGH)
- [ ] Modify `src/ollama.rs` to use `stream: true` in API requests
- [ ] Parse Server-Sent Events (SSE) from Ollama streaming endpoint
- [ ] Update `src/app.rs` to handle incremental message updates
- [ ] Show tokens as they arrive in the UI
- [ ] Handle stream errors and connection drops gracefully

**Technical Notes:**
- Ollama streaming returns newline-delimited JSON objects
- Each chunk has `{"message":{"content":"token"},"done":false}`
- Final chunk has `"done":true`
- Use `reqwest` streaming or `futures::Stream`

## Phase 2: Model Selector Dropdown (Priority: MEDIUM)
- [ ] Add function to query `GET /api/tags` for available models
- [ ] Create dropdown/menu widget in header area
- [ ] Display model names with sizes (e.g., "llama3.2:3b (2.0GB)")
- [ ] Save selected model to cosmic-config on change
- [ ] Load available models when popup opens

**Technical Notes:**
- API response: `{"models":[{"name":"llama3.2:3b","size":2000000000,...}]}`
- Use `cosmic::widget::dropdown` or `cosmic::widget::menu`

## Phase 3: Chat History Persistence (Priority: MEDIUM)
- [ ] Define chat history JSON schema
- [ ] Save history to `~/.local/share/cosmic-applet-ollama/history.json`
- [ ] Load history on applet startup
- [ ] Add "New Chat" button to clear and start fresh
- [ ] Implement max history limit (e.g., 100 messages)

**Technical Notes:**
- Use XDG directories via `dirs` crate or `std::env`
- Consider using cosmic-config for persistence instead

## Phase 4: Custom Icon (Priority: LOW)
- [ ] Design SVG icon representing AI/chat
- [ ] Follow COSMIC icon design guidelines
- [ ] Support both light and dark themes
- [ ] Update `resources/icon.svg`
- [ ] Test in panel at various sizes

## Success Criteria
- All clippy warnings resolved: `cargo clippy -- -D warnings`
- Code formatted: `cargo fmt --check`
- Build succeeds: `cargo build --release`
- CI pipeline passes
- Features work correctly when tested manually

## Exit Condition
When ALL phases are complete, mark them [x] and set EXIT_SIGNAL: true
