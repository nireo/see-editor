use crate::Position;
use std::io::{self, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

// A way to hold the width and height of the terminal window.
pub struct Size {
    pub width: u16,
    pub height: u16,
}

// The struct that handles all contant with terminal.
pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        // Get the terminal's size.
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    // Return a terminal's size.
    pub fn size(&self) -> &Size {
        &self.size
    }

    // Clear the terminal screen.
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    // Place the terminal cursor into the given position.
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    // Flush the terminal screen
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    // Read a key from standard input.
    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    // Hide the cursor in the terminal
    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    // Show the cursor in the terminal
    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    // Clear the current line in the terminal
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    // The the background color of the terminal
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }

    // The foreground color to a given color
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }

    // Reset the background color to the user's terminal's own color.
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    // Reset the foreground color to the user's terminal's own text color.
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }
}
