//! systeroid-tui

#![warn(missing_docs, clippy::unwrap_used)]

/// Main application.
pub mod app;
/// Command-line argument parser.
pub mod args;
/// Application commands.
pub mod command;
/// Error implementation.
pub mod error;
/// Event handling.
pub mod event;
/// User interface renderer.
pub mod ui;
/// Custom widgets.
pub mod widgets;

use crate::app::App;
use crate::args::Args;
use crate::command::Command;
use crate::error::Result;
use crate::event::{Event, EventHandler};
use std::io::Write;
use systeroid_core::cache::Cache;
use systeroid_core::config::Config;
use systeroid_core::sysctl::controller::Sysctl;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::terminal::Terminal;

/// Runs `systeroid-tui`.
pub fn run<Output: Write>(args: Args, output: Output) -> Result<()> {
    let output = output.into_raw_mode()?;
    let output = MouseTerminal::from(output);
    let output = AlternateScreen::from(output);
    let backend = TermionBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    let event_handler = EventHandler::new(args.tick_rate);
    let mut sysctl = Sysctl::init(Config::default())?;
    sysctl.update_docs_from_cache(args.kernel_docs.as_ref(), &Cache::init()?)?;
    let mut app = App::new(&mut sysctl);
    while app.running {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        match event_handler.next()? {
            Event::KeyPress(key) => {
                let command = Command::parse(key, app.is_input_mode());
                app.run_command(command)?;
            }
            Event::Tick => {
                app.tick();
            }
        }
    }

    Ok(())
}