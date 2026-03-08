# Claude Code - claude-cmd

## Project

Interactive TUI launcher for Claude Code written in Rust.

## Structure

```
claude-cmd/
├── Cargo.toml      # Config and dependencies
├── src/
│   └── main.rs     # Main code (TUI + logic)
├── README.md       # User documentation
└── CLAUDE.md       # This file
```

## Architecture

### Main Components (src/main.rs)

- `struct Option` - Represents a checkable option (label, arg, checked)
- `struct App` - Application state (options, current selection)
- `fn main()` - Terminal initialization, main loop, claude launch
- `fn run_app()` - Keyboard event loop
- `fn ui()` - Interface rendering with ratatui

### Execution Flow

1. Terminal initialization (raw mode, alternate screen)
2. Event loop (navigation, toggle, launch)
3. Terminal restoration
4. If "Launch" selected: execute `claude` with args

## Dependencies

- `crossterm` 0.27 - Keyboard events, terminal control
- `ratatui` 0.26 - TUI widgets, rendering

## Conventions

- Use `f.size()` (not `f.area()`) to get terminal size in ratatui 0.26
- Options are defined in `App::new()` - add new options there
- All code and documentation must be in English

## Build

```bash
cargo build --release
```

Binary: `target/release/claude-cmd.exe`

## Adding Options

To add a new option, modify `App::new()`:

```rust
Option {
    label: "Display name",
    arg: "--flag value",
    checked: false,
}
```
