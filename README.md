# COSMIC Ollama Applet

A panel applet for the [COSMIC](https://github.com/pop-os/cosmic-epoch) desktop that provides quick access to local AI assistance via [Ollama](https://ollama.com/).

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

## Features

- Chat with local Ollama models directly from your panel
- Automatic context gathering:
  - **Clipboard** - Copied text (Ctrl+C)
  - **Selection** - Highlighted text (no copy needed)
  - **System info** - OS, kernel, memory
  - **Recent errors** - Last 5 journal errors
- Pre-configured as a Pop!_OS/Linux assistant
- Fast responses with GPU acceleration

## Requirements

- [COSMIC Desktop](https://github.com/pop-os/cosmic-epoch) (Pop!_OS 24.04+ or other COSMIC-enabled distros)
- [Ollama](https://ollama.com/) installed and running
- `wl-clipboard` for clipboard integration

## Installation

### Install Dependencies

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Install clipboard tools
sudo apt install wl-clipboard

# Pull a model (choose one)
ollama pull phi3:mini      # Small, fast (~2GB)
ollama pull llama3.2:3b    # Smarter (~2GB)
ollama pull llama3.2       # Best quality (~4GB)
```

### Build from Source

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/paulwade/cosmic-applet-ollama.git
cd cosmic-applet-ollama
cargo build --release

# Install
sudo install -Dm0755 target/release/cosmic-applet-ollama /usr/bin/cosmic-applet-ollama
sudo install -Dm0644 resources/app.desktop /usr/share/applications/com.github.paulwade.cosmic-applet-ollama.desktop
```

### Add to Panel

1. Right-click on your COSMIC panel
2. Select "Add Applet"
3. Find "Ollama Assistant" and add it

## Usage

1. Click the applet icon in your panel
2. Type a question or request
3. For context-aware help:
   - **Copy text** (Ctrl+C) before asking - error messages, config files, code
   - **Select text** (highlight) - the applet reads primary selection too
4. Recent system errors are automatically included for troubleshooting

## Configuration

The default model is `phi3:mini`. To change it, edit `src/ollama.rs`:

```rust
pub const DEFAULT_MODEL: &str = "llama3.2:3b";
```

Then rebuild and reinstall.

## Project Structure

```
src/
├── main.rs      # Entry point
├── app.rs       # COSMIC applet UI and logic
├── ollama.rs    # Ollama API client
├── context.rs   # System context gathering
├── config.rs    # Configuration handling
└── i18n.rs      # Internationalization
```

## Development

```bash
# Run with logging
RUST_LOG=debug cargo run

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [System76](https://system76.com/) for COSMIC desktop and libcosmic
- [Ollama](https://ollama.com/) for making local LLMs accessible
