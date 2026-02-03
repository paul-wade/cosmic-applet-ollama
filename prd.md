# COSMIC Ollama Applet - Product Requirements Document

## Project Overview
A COSMIC desktop panel applet for chatting with local Ollama AI models with context awareness and web search.

## Current Status: v0.1.0
- [x] Basic chat UI with COSMIC styling
- [x] Message bubbles with theme colors
- [x] Configurable model via cosmic-config
- [x] Context gathering (clipboard, selection, system info, journal errors)
- [x] Web search integration (DuckDuckGo)
- [x] Honest system prompt about knowledge limits
- [x] CI/CD pipeline with GitHub Actions
- [x] Flatpak manifest

## Remaining Features

### Phase 1: Streaming Responses
- [ ] Implement streaming API calls to Ollama (`stream: true`)
- [ ] Show tokens as they arrive instead of waiting for full response
- [ ] Update UI progressively during generation
- [ ] Handle stream errors gracefully

### Phase 2: Model Selector Dropdown
- [ ] Query Ollama API for available models (`GET /api/tags`)
- [ ] Add dropdown/menu in header to select model
- [ ] Save selected model to cosmic-config
- [ ] Show model size/info in dropdown

### Phase 3: Chat History Persistence
- [ ] Save chat history to file (JSON in XDG data dir)
- [ ] Load previous chat on startup
- [ ] Add "New Chat" button to start fresh
- [ ] Limit history size to prevent bloat

### Phase 4: Better Icon
- [ ] Create custom SVG icon for panel
- [ ] Icon should represent AI/chat concept
- [ ] Follow COSMIC icon guidelines
- [ ] Support light/dark themes

## Technical Requirements
- All code must pass `cargo clippy -- -D warnings`
- All code must pass `cargo fmt --check`
- No new dependencies without justification
- Maintain compatibility with COSMIC desktop

## Success Criteria
- All features implemented and working
- CI passes on all commits
- No regressions in existing functionality
- Clean, documented code

## Completion Signal
Output `<promise>RALPH_COMPLETE</promise>` when ALL phases are done and tests pass.
