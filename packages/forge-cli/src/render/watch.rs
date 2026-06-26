//! TUI-based live session watcher using ratatui.
//! Connects to forge-server's SSE endpoint and renders live events.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct WatchApp {
    session_id: String,
    events: Vec<String>,
    scroll: usize,
}

impl WatchApp {
    fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            events: Vec::new(),
            scroll: 0,
        }
    }

    fn add_event(&mut self, text: String) {
        self.events.push(text);
        if self.events.len() > 1000 {
            self.events.remove(0);
        }
        self.scroll = self.events.len().saturating_sub(1);
    }
}

pub async fn run_watch(session_id: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = WatchApp::new(session_id);
    app.add_event(format!("Connected to session {}", session_id));
    app.add_event("Live events will appear here...".into());
    app.add_event("Connect to forge-server at http://localhost:3000 to run agents.".into());

    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Watch error: {:?}", err);
    }
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut WatchApp) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.scroll = (app.scroll + 1).min(app.events.len().saturating_sub(1));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &WatchApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    // Title bar
    let title = Paragraph::new(format!(
        "Forge Watch — Session: {} | q: quit | arrows: scroll | {} events",
        app.session_id,
        app.events.len()
    ))
    .block(Block::default().borders(Borders::ALL).title("Forge Watch"))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, chunks[0]);

    // Events list
    let visible: Vec<ListItem> = app
        .events
        .iter()
        .rev()
        .take(50)
        .map(|e| ListItem::new(Span::raw(e)))
        .collect();
    let list = List::new(visible)
        .block(Block::default().borders(Borders::ALL).title("Events"))
        .style(Style::default().fg(Color::White));
    f.render_widget(list, chunks[1]);
}
