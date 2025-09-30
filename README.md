# Tui-chat

A Rust crate providing terminal chat widgets for ratatui applications. Includes `ChatArea` for displaying messages and `InputArea` for multiline text input, along with a complete `ChatApp` for quick prototyping.

## Features

- **Multiline Input**: Type messages with line breaks using Shift+Enter.
- **Scrollable Chat History**: Navigate through messages with Page Up/Down or mouse wheel.
- **Simulated AI Responses**: Automatically responds to user messages for demonstration.
- **Keyboard Navigation**: Full cursor movement support in input area (arrow keys, etc.).
- **Cross-Platform**: Works on Windows, macOS, and Linux.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tui-chat = "0.1.0"
ratatui = "0.29.0"
crossterm = "0.29.0"
textwrap = "0.16.0"
```

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

## Usage

### As a Library

```rust
use tui_chat::{ChatArea, InputArea, ChatMessage};

let mut chat_area = ChatArea::new();
let mut input_area = InputArea::new();

// Add a message
chat_area.add_message(ChatMessage {
    sender: "User".to_string(),
    content: "Hello!".to_string(),
});

// In your render loop
chat_area.render(frame, chat_rect);
input_area.render(frame, input_rect);
```

### Running the Example

To see a full chat application, run the included example:

```bash
cargo run --example chat_app
```

### Keybindings (for ChatApp example)

- **Enter**: Send message
- **Shift+Enter**: New line in input
- **Page Up/Down**: Scroll chat history
- **Arrow Keys**: Navigate cursor in input area
- **Backspace**: Delete character
- **Ctrl+C** or **Esc**: Quit application

## Dependencies

- [crossterm](https://crates.io/crates/crossterm): Cross-platform terminal manipulation
- [ratatui](https://crates.io/crates/ratatui): Terminal UI framework

## License

This project is licensed under the MIT License.