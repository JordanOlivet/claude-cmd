# claude-cmd

Interactive launcher for Claude Code with TUI (Terminal User Interface).

## Features

- Visual interface with checkboxes
- Option selection before launch
- Intuitive keyboard navigation

## Available Options

| Option | Generated Argument |
|--------|-------------------|
| Use Opus 4.5 | `--model claude-opus-4-5-20251101` |
| Skip permissions | `--dangerously-skip-permissions` |

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) installed

### Build

```bash
cargo build --release
```

Binary will be in `target/release/claude-cmd.exe`.

### Add to PATH

Copy `claude-cmd.exe` to a folder in PATH, or add `target/release` to PATH.

## Usage

```bash
claude-cmd
```

### Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Space` | Toggle option |
| `Enter` | Toggle option / Launch |
| `q` / `Esc` | Quit |

### Interface

```
┌─ Claude Code Launcher ───────────────┐
│                                      │
│  > [x] Use Opus 4.5                  │
│    [ ] Skip permissions              │
│                                      │
│    >>> Launch Claude <<<             │
│                                      │
│  ↑/↓: navigate  Enter/Space: toggle  │
└──────────────────────────────────────┘
```

## Dependencies

- `crossterm` 0.27 - Terminal/keyboard handling
- `ratatui` 0.26 - TUI interface

## License

MIT
