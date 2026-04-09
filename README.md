# claude-cmd

Interactive launcher for Claude Code with TUI (Terminal User Interface).

## Features

- Visual interface with checkboxes
- Option selection before launch
- Intuitive keyboard navigation

## Available Options

| Option | Generated Argument |
|--------|-------------------|
| Skip permissions | `--dangerously-skip-permissions` |
| Use Opus 4.5 | `--model claude-opus-4-5-20251101` |

## Installation

### From GitHub Releases (recommended)

Download the latest binary from the [Releases](https://github.com/JordanOlivet/claude-cmd/releases) page.

#### Windows

```powershell
# Download the binary
curl -Lo claude-cmd.exe https://github.com/JordanOlivet/claude-cmd/releases/latest/download/claude-cmd-windows-x86_64.exe

# Move it to a directory in your PATH
move claude-cmd.exe C:\Users\%USERNAME%\.local\bin\
```

#### Linux

```bash
# Download the binary
curl -Lo claude-cmd https://github.com/JordanOlivet/claude-cmd/releases/latest/download/claude-cmd-linux-x86_64

# Make it executable and move it to your PATH
chmod +x claude-cmd
sudo mv claude-cmd /usr/local/bin/
```

#### macOS

```bash
# Download the binary (Apple Silicon)
curl -Lo claude-cmd https://github.com/JordanOlivet/claude-cmd/releases/latest/download/claude-cmd-macos-arm64

# Or for Intel Macs
# curl -Lo claude-cmd https://github.com/JordanOlivet/claude-cmd/releases/latest/download/claude-cmd-macos-x86_64

# Make it executable and move it to your PATH
chmod +x claude-cmd
sudo mv claude-cmd /usr/local/bin/
```

### From source

Requires [Rust](https://rustup.rs/).

```bash
cargo build --release
```

Binary will be in `target/release/claude-cmd` (or `claude-cmd.exe` on Windows). Copy it to a directory in your PATH.

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
│  > [x] Skip permissions              │
│    [ ] Use Opus 4.5                  │
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