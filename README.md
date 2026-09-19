# claude-cmd

Interactive launcher for Claude Code with TUI (Terminal User Interface).

## Features

- Launch button first: press `Enter` to start Claude immediately with your saved options
- Choices are persisted between launches (config file)
- Toggle options (checkboxes) and exclusive choices (radio groups) for model and session
- Live preview of the generated `claude` command
- Intuitive keyboard navigation

## Available Options

### Toggles

| Option | Generated Argument |
|--------|-------------------|
| Skip permissions | `--dangerously-skip-permissions` |

### Model (exclusive choice)

| Choice | Generated Argument |
|--------|-------------------|
| Default model | *(none)* |
| Fable 5 (1M context) | `--model claude-fable-5[1m]` |
| Opus 4.8 (1M context) | `--model claude-opus-4-8[1m]` |
| Opus 4.6 | `--model claude-opus-4-6` |
| Opus 4.5 | `--model claude-opus-4-5-20251101` |

### Session (exclusive choice)

| Choice | Generated Argument |
|--------|-------------------|
| New session | *(none)* |
| Continue last session | `--continue` |
| Resume a session | `--resume` |

## Installation

### From GitHub Releases (recommended)

Download the latest binary from the [Releases](https://github.com/JordanOlivet/claude-cmd/releases) page.

#### Windows

```powershell
# Download the binary
curl.exe -Lo claude-cmd.exe https://github.com/JordanOlivet/claude-cmd/releases/latest/download/claude-cmd-windows-x86_64.exe

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

The cursor starts on the launch button: press `Enter` to launch with your last-used options, or navigate down to adjust them first.

### Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Space` | Toggle option / select choice |
| `Enter` | Launch Claude (from anywhere) |
| `q` / `Esc` | Quit without launching |

### Interface

```
┌─ Claude Code Launcher ─────────────────────┐
│                                            │
│  > >>> Launch Claude <<<                   │
│                                            │
│    [x] Skip permissions                    │
│                                            │
│  Model                                     │
│    ( ) Default model                       │
│    (•) Fable 5 (1M context)                │
│    ( ) Opus 4.8 (1M context)               │
│    ( ) Opus 4.6                            │
│    ( ) Opus 4.5                            │
│                                            │
│  Session                                   │
│    (•) New session                         │
│    ( ) Continue last session               │
│    ( ) Resume a session                    │
│                                            │
│ $ claude --dangerously-skip-permissions ...│
│ ↑/↓: navigate  Space: select  Enter: launch│
└────────────────────────────────────────────┘
```

## Configuration File

Choices are saved when you launch Claude and restored on the next start:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\claude-cmd\config.json` |
| Linux | `~/.config/claude-cmd/config.json` |
| macOS | `~/Library/Application Support/claude-cmd/config.json` |

Options are stored by label, so entries from older versions are ignored gracefully and new options fall back to their defaults.

## Dependencies

- `crossterm` 0.27 - Terminal/keyboard handling
- `ratatui` 0.26 - TUI interface
- `serde` / `serde_json` - Config persistence
- `dirs` - Cross-platform config directory

## License

MIT
