# truename: v0.0.1 — hello, player

The smallest possible thing that earns the title "I'm building a roguelike in Rust." No items, no names, no map — just a terminal that opens cleanly, a `@` you can move around inside a border, a status bar that hints at what's coming, and a clean exit on `q`. Ship this, then iterate.

## scope

- Opens an alternate-screen terminal UI with a bordered "truename" frame
- An `@` renders inside, in cool-purple
- Arrow keys *or* hjkl move the `@`
- Status bar at the bottom showing `you are (?)` (placeholder for the name mechanic)
- `q` or `esc` exits cleanly

That's it. Should be ~80 lines of code total.

## setup

```bash
cargo new truename
cd truename
cargo add ratatui crossterm
```

This scaffolds the project and pulls in ratatui (terminal UI) and crossterm (the cross-platform terminal backend ratatui sits on). Cargo will write the dependencies into `Cargo.toml` automatically.

## src/main.rs

Replace the contents of `src/main.rs` with this:

```rust
use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
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
        Self { player_x: 5, player_y: 3 }
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
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
```

## what's happening here

A quick tour of the Rust idioms you just touched, in case any of them feel mysterious:

- **`?` operator**: appears all over `main()` — it means "if this `Result` is an `Err`, return it from the enclosing function; otherwise unwrap the `Ok`." It's the ergonomic version of try/catch for `Result` and `Option`. You'll use this constantly.
- **`struct App` + `impl App`**: the standard Rust pattern for "a thing and the methods on it." Methods go in `impl` blocks, not inside the struct definition.
- **`Self { player_x: 5, .. }`**: `Self` is shorthand for the type you're inside an `impl` for. Reads cleanly.
- **`run_app<B: Backend>`**: generic function with a trait bound — `B` can be any type that implements the `Backend` trait. This lets `run_app` work with the real terminal *or* a test backend without changing code. It's also why ratatui's API is so composable.
- **`saturating_sub` / `saturating_add`**: these are `u16` methods that clamp at 0 and `u16::MAX` instead of wrapping or panicking. Without them, moving left at x=0 would underflow (since `u16` is unsigned) and crash you. This is the kind of thing Rust makes you think about that other languages just silently break on.
- **`if let Event::Key(key) = event::read()?`**: pattern matching in a conditional. Only enters the block if `event::read()` returned an `Event::Key` variant.
- **`match key.code`**: exhaustive pattern matching. Multiple patterns can share an arm with `|`. The `_ => {}` at the end catches everything else and does nothing — and the compiler will error if you forget to handle a case, which is *delightful*.
- **`&app` in `ui(f, &app)`**: borrowing immutably. `ui` doesn't need to own or mutate the app, just look at it. This is the borrow checker setting expectations: ownership stays with `run_app`, `ui` gets a temporary read-only view.

The piece that'll feel weirdest if you've come from TS/Java is *who owns what when*. `terminal` is owned by `main`, mutably borrowed into `run_app`, mutably borrowed again per-frame into the closure passed to `terminal.draw`. None of this needs annotation because it's all stack-scoped and the compiler figures it out. The moment you start passing things across threads or storing references in structs, the lifetimes get explicit. Don't worry about it yet.

## run it

```bash
cargo run
```

First compile will take a minute (it's pulling and building ratatui + its dependency tree). Subsequent `cargo run` invocations will be fast because incremental compilation kicks in.

You should see: a bordered "truename" frame fills your terminal, a purple `@` near the top-left, a status bar at the bottom. Arrow keys or hjkl move the `@`. Press `q` or `esc` to quit cleanly.

If something looks weird — characters not rendering, colors off, frame stuck — first check you're in Windows Terminal (not the legacy console host). That's the usual culprit on Windows.

## next: v0.0.2

The minimum next step that earns "this is starting to feel like a game" is the **name scroll**:

- Add an `Item` enum with one variant: `NameScroll(String)`.
- Place a `?` on the map at a fixed location representing a scroll holding the name `"sora"` (or whatever — pick something you'd actually want to be called).
- When the player walks onto the `?`, move it from the map into a `Vec<Item>` on the App (inventory).
- Add a `w` keybind that wears the most recent name scroll: replaces the `(?)` in the status bar with the name, and bumps a visible stat (let's say `clarity: 1`) to start hinting at name effects.

That's the v0.0.2 scope. Once you can pick up a scroll and wear a name and see the world acknowledge it, you have the *entire core loop* of truename in miniature. Everything after that is content and elaboration.
