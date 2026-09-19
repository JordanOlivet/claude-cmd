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

- `struct ToggleOption` - Independent checkbox option (label, arg, checked)
- `struct RadioChoice` / `struct RadioGroup` - Exclusive choices (Model, Session); a choice with an empty `arg` produces no flag
- `enum Focus` - Resolves the flat focus index into Button / Toggle / Choice
- `struct App` - Application state (toggles, groups, focus index; focus 0 = launch button)
- `struct SavedConfig` + `config_path()` - Persistence of choices as JSON, keyed by label
- `fn main()` - Terminal initialization, background update check, main loop, config save, claude launch
- `fn run_app()` - Keyboard event loop (`Space` = toggle/select, `Enter` = launch from anywhere); polls events with a 200ms timeout and drains the update-check channel so the notice can appear without a key press
- Update check - spawned as a thread before the TUI starts (never blocks startup or launch); on a newer GitHub release, a yellow notice line suggests `claude-cmd update`
- `fn ui()` - Interface rendering with ratatui (launch button first, sections, command preview)

### Execution Flow

1. Terminal initialization (raw mode, alternate screen)
2. `App::new()` builds defaults, then `apply_saved_config()` restores saved choices by label
3. Event loop (navigation, toggle/select, launch); cursor starts on the launch button
4. Terminal restoration
5. If launched: `save_config()` writes choices, then execute `claude` with args

### Config Persistence

- Path: `dirs::config_dir()/claude-cmd/config.json` (e.g. `%APPDATA%\claude-cmd\config.json` on Windows)
- Saved on launch only; load/parse errors are silently ignored (launcher must never fail on config issues)
- Keys are option labels / group names, so renaming a label resets that entry to its default

## Dependencies

- `crossterm` 0.27 - Keyboard events, terminal control
- `ratatui` 0.26 - TUI widgets, rendering
- `serde` / `serde_json` - Config file (de)serialization
- `dirs` - Cross-platform config directory

## Conventions

- Use `f.size()` (not `f.area()`) to get terminal size in ratatui 0.26
- Options are defined in `App::new()` - add new toggles or radio choices there
- All code and documentation must be in English

## Build

```bash
cargo build --release
```

Binary: `target/release/claude-cmd.exe`

## Adding Options

To add a toggle, append to `toggles` in `App::new()`:

```rust
ToggleOption {
    label: "Display name",
    arg: "--flag value",
    checked: false,
}
```

To add a model or session choice, append a `RadioChoice` to the matching `RadioGroup`:

```rust
RadioChoice {
    label: "Display name",
    arg: "--model some-model",
}
```

The UI box height is computed from the item count, so no layout change is needed.
