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
                    label: "Use Opus 4.5",
                    arg: "--model claude-opus-4-5-20251101",
                    checked: true,
                },
                Option {
                    label: "Skip permissions",
                    arg: "--dangerously-skip-permissions",
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

fn main() -> io::Result<()> {
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
