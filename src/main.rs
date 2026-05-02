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
use std::io::{self, stdout};
use std::process::Command;

struct Option {
    label: &'static str,
    arg: &'static str,
    checked: bool,
}

struct App {
    options: Vec<Option>,
    selected: usize,
    launch_selected: bool,
}

impl App {
    fn new() -> Self {
        Self {
            options: vec![
                Option {
                    label: "Skip permissions",
                    arg: "--dangerously-skip-permissions",
                    checked: true,
                },
                Option {
                    label: "Use Opus 4.6",
                    arg: "--model claude-opus-4-6",
                    checked: true,
                },
                Option {
                    label: "Use Opus 4.5",
                    arg: "--model claude-opus-4-5-20251101",
                    checked: false,
                },
            ],
            selected: 0,
            launch_selected: false,
        }
    }

    fn move_up(&mut self) {
        if self.launch_selected {
            self.launch_selected = false;
            self.selected = self.options.len() - 1;
        } else if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.launch_selected {
            return;
        }
        if self.selected < self.options.len() - 1 {
            self.selected += 1;
        } else {
            self.launch_selected = true;
        }
    }

    fn toggle(&mut self) {
        if !self.launch_selected {
            self.options[self.selected].checked = !self.options[self.selected].checked;
        }
    }

    fn build_command(&self) -> Vec<String> {
        let mut args = Vec::new();
        for opt in &self.options {
            if opt.checked {
                args.extend(opt.arg.split_whitespace().map(String::from));
            }
        }
        args
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

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Ok(true) = result {
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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Char(' ') => app.toggle(),
                KeyCode::Enter => {
                    if app.launch_selected {
                        return Ok(true);
                    } else {
                        app.toggle();
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();

    let box_width = 42;
    let box_height = 10;
    let x = (size.width.saturating_sub(box_width)) / 2;
    let y = (size.height.saturating_sub(box_height)) / 2;

    let area = Rect::new(x, y, box_width.min(size.width), box_height.min(size.height));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Claude Code Launcher ")
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let mut lines = Vec::new();
    for (i, opt) in app.options.iter().enumerate() {
        let checkbox = if opt.checked { "[x]" } else { "[ ]" };
        let prefix = if !app.launch_selected && i == app.selected {
            ">"
        } else {
            " "
        };
        let style = if !app.launch_selected && i == app.selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(
            format!("  {} {} {}", prefix, checkbox, opt.label),
            style,
        )));
    }

    lines.push(Line::from(""));

    let launch_style = if app.launch_selected {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let launch_prefix = if app.launch_selected { ">" } else { " " };
    lines.push(Line::from(Span::styled(
        format!("  {} >>> Launch Claude <<<", launch_prefix),
        launch_style,
    )));

    let options_paragraph = Paragraph::new(lines);
    f.render_widget(options_paragraph, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::styled(": navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
        Span::styled(": toggle  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::styled(": quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(help, chunks[3]);
}
