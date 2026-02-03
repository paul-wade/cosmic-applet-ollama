# Ralph Loop Prompt - COSMIC Ollama Applet

You are implementing features for the COSMIC Ollama Applet. Read `prd.md` for the full requirements.

## Instructions

1. **Check Progress**: Read `prd.md` and check which items are marked `[x]` (done) vs `[ ]` (todo)

2. **Work on Next Item**: Implement the next unchecked item in order:
   - Phase 1: Streaming Responses
   - Phase 2: Model Selector Dropdown
   - Phase 3: Chat History Persistence
   - Phase 4: Better Icon

3. **After Each Change**:
   - Run `cargo fmt`
   - Run `cargo clippy -- -D warnings`
   - Run `cargo build --release`
   - Fix any errors before proceeding

4. **Update PRD**: After completing each item, mark it `[x]` in `prd.md`

5. **Commit**: After each phase, commit with a descriptive message

6. **Completion**: When ALL phases are done and verified working:
   - Ensure all items in prd.md are marked `[x]`
   - Run final build and clippy check
   - Output: `<promise>RALPH_COMPLETE</promise>`

## Key Files
- `src/app.rs` - Main UI and application logic
- `src/ollama.rs` - Ollama API client
- `src/context.rs` - Context gathering
- `src/config.rs` - Configuration
- `src/web.rs` - Web search
- `resources/icon.svg` - Panel icon

## Notes
- Use COSMIC widgets and theme system
- Follow existing code patterns
- Keep responses concise (small panel UI)
- Test changes before marking complete
