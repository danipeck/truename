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
        Self { 
            player_x: 5, 
            player_y: 3 
        }
    }
}

fn main() {
    println!("Hello, world!");
}
