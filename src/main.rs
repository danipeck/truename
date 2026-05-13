use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

/// Everything the game knows about the world right now.
/// For v0.0.1 that's just where the player is.
struct App {
    player_x: u16,
    player_y: u16,
}

impl App {
    fn new() -> Self {
        Self { 
            player_x: 5, 
            player_y: 3 
        }
    }
}

fn main() -> io::Result<()> {
    // --- Terminal setup ---
    // Raw mode = we get individual keypresses, not line-buffered input.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Alternate screen = we don't clobber the user's shell scrollback.
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- Run the game ---
    let result = run_app(&mut terminal);

    // --- Terminal teardown (runs even if run_app errored) ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Up    | KeyCode::Char('k') => app.player_y = app.player_y.saturating_sub(1),
                KeyCode::Down  | KeyCode::Char('j') => app.player_y = app.player_y.saturating_add(1),
                KeyCode::Left  | KeyCode::Char('h') => app.player_x = app.player_x.saturating_sub(1),
                KeyCode::Right | KeyCode::Char('l') => app.player_x = app.player_x.saturating_add(1),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    // Split the screen: main game area on top, status bar on the bottom.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    let game_area = chunks[0];
    let status_area = chunks[1];

    // --- Game area: bordered frame with the player inside ---
    let game_block = Block::default().title(" truename ").borders(Borders::ALL);
    let inner = game_block.inner(game_area);
    f.render_widget(game_block, game_area);

    // Build the playfield as a Vec<Line>: a grid of spaces with '@' at the player's position.
    let mut lines: Vec<Line> = Vec::new();
    for y in 0..inner.height {
        let mut row = String::new();
        for x in 0..inner.width {
            if x == app.player_x && y == app.player_y {
                row.push('@');
            } else {
                row.push(' ');
            }
        }
        lines.push(Line::from(row));
    }

    // Tokyo Night purple-ish for the @.
    let purple = Color::Rgb(187, 154, 247);
    let playfield = Paragraph::new(lines).style(Style::default().fg(purple));
    f.render_widget(playfield, inner);

    // --- Status bar: placeholder for the name mechanic ---
    let status = Paragraph::new("you are (?)  ·  hjkl/arrows to move  ·  q to quit")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, status_area);
}
