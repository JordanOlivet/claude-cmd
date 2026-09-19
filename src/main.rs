use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::collections::HashMap;
use std::io::{self, stdout};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

struct ToggleOption {
    label: &'static str,
    arg: &'static str,
    checked: bool,
}

struct RadioChoice {
    label: &'static str,
    arg: &'static str,
}

struct RadioGroup {
    name: &'static str,
    choices: Vec<RadioChoice>,
    selected: usize,
}

enum Focus {
    Button,
    Toggle(usize),
    Choice(usize, usize),
}

struct App {
    toggles: Vec<ToggleOption>,
    groups: Vec<RadioGroup>,
    focus: usize,
    update_notice: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SavedConfig {
    #[serde(default)]
    toggles: HashMap<String, bool>,
    #[serde(default)]
    radios: HashMap<String, String>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("claude-cmd").join("config.json"))
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            toggles: vec![ToggleOption {
                label: "Skip permissions",
                arg: "--dangerously-skip-permissions",
                checked: true,
            }],
            groups: vec![
                RadioGroup {
                    name: "Model",
                    choices: vec![
                        RadioChoice {
                            label: "Default model",
                            arg: "",
                        },
                        RadioChoice {
                            label: "Fable 5 (1M context)",
                            arg: "--model claude-fable-5[1m]",
                        },
                        RadioChoice {
                            label: "Opus 4.8 (1M context)",
                            arg: "--model claude-opus-4-8[1m]",
                        },
                        RadioChoice {
                            label: "Opus 4.6",
                            arg: "--model claude-opus-4-6",
                        },
                        RadioChoice {
                            label: "Opus 4.5",
                            arg: "--model claude-opus-4-5-20251101",
                        },
                    ],
                    selected: 1,
                },
                RadioGroup {
                    name: "Session",
                    choices: vec![
                        RadioChoice {
                            label: "New session",
                            arg: "",
                        },
                        RadioChoice {
                            label: "Continue last session",
                            arg: "--continue",
                        },
                        RadioChoice {
                            label: "Resume a session",
                            arg: "--resume",
                        },
                    ],
                    selected: 0,
                },
            ],
            focus: 0,
            update_notice: None,
        };
        app.apply_saved_config();
        app
    }

    fn item_count(&self) -> usize {
        1 + self.toggles.len() + self.groups.iter().map(|g| g.choices.len()).sum::<usize>()
    }

    fn focus_target(&self) -> Focus {
        let mut idx = self.focus;
        if idx == 0 {
            return Focus::Button;
        }
        idx -= 1;
        if idx < self.toggles.len() {
            return Focus::Toggle(idx);
        }
        idx -= self.toggles.len();
        for (gi, group) in self.groups.iter().enumerate() {
            if idx < group.choices.len() {
                return Focus::Choice(gi, idx);
            }
            idx -= group.choices.len();
        }
        Focus::Button
    }

    fn move_up(&mut self) {
        if self.focus > 0 {
            self.focus -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.focus + 1 < self.item_count() {
            self.focus += 1;
        }
    }

    fn activate(&mut self) {
        match self.focus_target() {
            Focus::Button => {}
            Focus::Toggle(i) => self.toggles[i].checked = !self.toggles[i].checked,
            Focus::Choice(g, c) => self.groups[g].selected = c,
        }
    }

    fn build_command(&self) -> Vec<String> {
        let mut args = Vec::new();
        for opt in &self.toggles {
            if opt.checked {
                args.extend(opt.arg.split_whitespace().map(String::from));
            }
        }
        for group in &self.groups {
            let arg = group.choices[group.selected].arg;
            if !arg.is_empty() {
                args.extend(arg.split_whitespace().map(String::from));
            }
        }
        args
    }

    fn preview(&self) -> String {
        let mut cmd = String::from("claude");
        for arg in self.build_command() {
            cmd.push(' ');
            cmd.push_str(&arg);
        }
        cmd
    }

    // Saved options are matched by label so reordering or adding options
    // in a future version does not corrupt restored choices.
    fn apply_saved_config(&mut self) {
        let Some(path) = config_path() else { return };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };
        let saved: SavedConfig = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return,
        };
        for toggle in &mut self.toggles {
            if let Some(&checked) = saved.toggles.get(toggle.label) {
                toggle.checked = checked;
            }
        }
        for group in &mut self.groups {
            if let Some(label) = saved.radios.get(group.name) {
                if let Some(i) = group.choices.iter().position(|c| c.label == label) {
                    group.selected = i;
                }
            }
        }
    }

    fn save_config(&self) {
        let Some(path) = config_path() else { return };
        let saved = SavedConfig {
            toggles: self
                .toggles
                .iter()
                .map(|t| (t.label.to_string(), t.checked))
                .collect(),
            radios: self
                .groups
                .iter()
                .map(|g| (g.name.to_string(), g.choices[g.selected].label.to_string()))
                .collect(),
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&saved) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("Warning: could not save config: {}", e);
                }
            }
            Err(e) => eprintln!("Warning: could not serialize config: {}", e),
        }
    }
}

#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

fn get_asset_name() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("claude-cmd-windows-x86_64.exe"),
        ("linux", "x86_64") => Ok("claude-cmd-linux-x86_64"),
        ("macos", "x86_64") => Ok("claude-cmd-macos-x86_64"),
        ("macos", "aarch64") => Ok("claude-cmd-macos-arm64"),
        (os, arch) => Err(format!("Unsupported platform: {}-{}", os, arch)),
    }
}

fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/JordanOlivet/claude-cmd/releases/latest";
    let body = ureq::get(url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "claude-cmd-updater")
        .call()?
        .body_mut()
        .read_to_string()?;
    let release: GitHubRelease = serde_json::from_str(&body)?;
    Ok(release)
}

fn download_asset(url: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let bytes = ureq::get(url)
        .header("User-Agent", "claude-cmd-updater")
        .call()?
        .body_mut()
        .read_to_vec()?;
    let tmp_path = std::env::temp_dir().join("claude-cmd-update");
    std::fs::write(&tmp_path, &bytes)?;
    Ok(tmp_path)
}

fn run_update() -> io::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: v{}", current_version);
    println!("Checking for updates...");

    let asset_name = match get_asset_name() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let release = match fetch_latest_release() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error checking for updates: {}", e);
            std::process::exit(1);
        }
    };

    let latest_version = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
    if latest_version == current_version {
        println!("Already up to date (v{}).", current_version);
        return Ok(());
    }
    println!("New version available: v{} -> v{}", current_version, latest_version);

    let asset = match release.assets.iter().find(|a| a.name == asset_name) {
        Some(a) => a,
        None => {
            eprintln!(
                "Error: No asset '{}' found in release v{}. Available: {}",
                asset_name,
                latest_version,
                release
                    .assets
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            std::process::exit(1);
        }
    };

    println!("Downloading {}...", asset.name);
    let tmp_path = match download_asset(&asset.browser_download_url) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error downloading update: {}", e);
            std::process::exit(1);
        }
    };

    println!("Installing...");
    match self_replace::self_replace(&tmp_path) {
        Ok(()) => {
            std::fs::remove_file(&tmp_path).ok();
            println!("Successfully updated to v{}!", latest_version);
        }
        Err(e) => {
            std::fs::remove_file(&tmp_path).ok();
            eprintln!("Error replacing binary: {}", e);
            eprintln!("You may need to run with elevated permissions.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "update" => return run_update(),
            "--version" | "-V" => {
                println!("claude-cmd v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            other => {
                eprintln!("Unknown command: {}", other);
                std::process::exit(1);
            }
        }
    }

    // Check for updates in the background so the TUI starts instantly;
    // the result is picked up by the event loop when (and if) it arrives.
    let (update_tx, update_rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(release) = fetch_latest_release() {
            let latest = release
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&release.tag_name)
                .to_string();
            if latest != env!("CARGO_PKG_VERSION") {
                let _ = update_tx.send(latest);
            }
        }
    });

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app, &update_rx);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Ok(true) = result {
        app.save_config();
        let args = app.build_command();
        let status = Command::new("claude").args(&args).status();

        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    std::process::exit(exit_status.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("Failed to launch claude: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    update_rx: &mpsc::Receiver<String>,
) -> io::Result<bool> {
    loop {
        if app.update_notice.is_none() {
            if let Ok(latest) = update_rx.try_recv() {
                app.update_notice = Some(latest);
            }
        }

        terminal.draw(|f| ui(f, app))?;

        // Poll instead of blocking so the update notice can appear
        // without waiting for a key press.
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Char(' ') => app.activate(),
                KeyCode::Enter => return Ok(true),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();

    // Content: button + (blank + toggles) + per group (blank + header + choices)
    let content_height = 1
        + 1
        + app.toggles.len()
        + app
            .groups
            .iter()
            .map(|g| 2 + g.choices.len())
            .sum::<usize>();
    // + top padding, preview, help, 2 borders
    let mut box_height = (content_height + 5) as u16;
    let preview = app.preview();
    let mut box_width = (preview.len() as u16 + 6).max(46);

    let notice = app.update_notice.as_ref().map(|latest| {
        format!(
            "Update available: v{} -> v{} - run 'claude-cmd update'",
            env!("CARGO_PKG_VERSION"),
            latest
        )
    });
    if let Some(notice) = &notice {
        box_height += 1;
        box_width = box_width.max(notice.len() as u16 + 4);
    }

    let x = (size.width.saturating_sub(box_width)) / 2;
    let y = (size.height.saturating_sub(box_height)) / 2;

    let area = Rect::new(x, y, box_width.min(size.width), box_height.min(size.height));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Claude Code Launcher ")
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ];
    if notice.is_some() {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Length(1));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let focus_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let mut lines = Vec::new();
    let mut idx = 0usize;

    let launch_style = if app.focus == idx {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let launch_prefix = if app.focus == idx { ">" } else { " " };
    lines.push(Line::from(Span::styled(
        format!("  {} >>> Launch Claude <<<", launch_prefix),
        launch_style,
    )));
    idx += 1;

    lines.push(Line::from(""));
    for opt in &app.toggles {
        let checkbox = if opt.checked { "[x]" } else { "[ ]" };
        let focused = app.focus == idx;
        let prefix = if focused { ">" } else { " " };
        let style = if focused { focus_style } else { Style::default() };
        lines.push(Line::from(Span::styled(
            format!("  {} {} {}", prefix, checkbox, opt.label),
            style,
        )));
        idx += 1;
    }

    for group in &app.groups {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", group.name),
            Style::default().fg(Color::DarkGray),
        )));
        for (ci, choice) in group.choices.iter().enumerate() {
            let marker = if group.selected == ci { "(•)" } else { "( )" };
            let focused = app.focus == idx;
            let prefix = if focused { ">" } else { " " };
            let style = if focused { focus_style } else { Style::default() };
            lines.push(Line::from(Span::styled(
                format!("  {} {} {}", prefix, marker, choice.label),
                style,
            )));
            idx += 1;
        }
    }

    let options_paragraph = Paragraph::new(lines);
    f.render_widget(options_paragraph, chunks[1]);

    let preview_paragraph = Paragraph::new(Line::from(Span::styled(
        format!(" $ {}", preview),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(preview_paragraph, chunks[2]);

    if let Some(notice) = &notice {
        let notice_paragraph = Paragraph::new(Line::from(Span::styled(
            format!(" {}", notice),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        f.render_widget(notice_paragraph, chunks[3]);
    }

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::styled(": navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::styled(": select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(": launch  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
        Span::styled(": quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(help, chunks[chunks.len() - 1]);
}
