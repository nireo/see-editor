use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::io::stdout;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;
use termion::raw::IntoRawMode;

const STATUS_BG_COLOR: color::Rgb = color::Rgb(255, 255, 255);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

#[derive(PartialEq)]
enum EditorMode {
    Insert,
    View,
}

pub struct Editor {
    quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    editor_mode: EditorMode,
    documents: Vec<Document>,
    open_document: usize,
}

impl Editor {
    pub fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        loop {
            if let Err(error) = self.refresh_editor() {
                end(error);
            }

            if self.quit {
                break;
            }

            if let Err(error) = self.process_press() {
                end(error);
            }
        }
    }

    fn change_mode(&mut self, to_change: EditorMode) {
        self.editor_mode = to_change;
    }

    fn handle_file_save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("save stopped".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("file saved".to_string());
        } else {
            self.status_message = StatusMessage::from("error writing file".to_string());
        }
    }

    // Check if the wants to exit without saving changes. If the user wants to exit without saving,
    // signal a quit request, so that when refreshing the editor the session will end.
    fn check_exit_without_saving(&mut self) {
        // If not edited just don't do anything and quit the program
        if !self.document.edited() {
            self.quit = true;
            return;
        }

        let action = self
            .prompt("exit without saving? (yes/no)", |_, _, _| {})
            .unwrap_or(None);

        match action {
            Some(action) => {
                if action.to_string() == "yes" || action.to_string() == "y" {
                    self.quit = true;
                    return;
                }
            }
            None => return,
        }
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt("search: ", |editor, key, query| {
                let mut moved = false;
                match key {
                    Key::Right | Key::Down => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(Key::Right);
                        moved = true;
                    }
                    Key::Left | Key::Up => direction = SearchDirection::Backward,
                    _ => direction = SearchDirection::Forward,
                }
                if let Some(position) =
                    editor
                        .document
                        .find(&query, &editor.cursor_position, direction)
                {
                    editor.cursor_position = position;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(Key::Left);
                }
                editor.document.highlight(Some(query));
            })
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }

        self.document.highlight(None);
    }

    fn process_press(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        // There are different keybindings depending on which mode you're in, so check which
        // keybindings to use.
        if self.editor_mode == EditorMode::View {
            // EditorMode::View is similar to vim's normal mode
            match pressed_key {
                Key::Char('i') => self.change_mode(EditorMode::Insert),
                Key::Char('j') => self.move_cursor(Key::Down),
                Key::Char('h') => self.move_cursor(Key::Left),
                Key::Char('k') => self.move_cursor(Key::Up),
                Key::Char('l') => self.move_cursor(Key::Right),
                Key::Ctrl('q') => self.check_exit_without_saving(),
                Key::Ctrl('s') => self.handle_file_save(),
                Key::Ctrl('f') => self.search(),
                Key::Ctrl('n') => self.open_new_file(),
                _ => (),
            }
        } else if self.editor_mode == EditorMode::Insert {
            match pressed_key {
                Key::Ctrl('q') => self.check_exit_without_saving(),
                Key::Ctrl('s') => self.handle_file_save(),
                Key::Ctrl('f') => self.search(),
                Key::Ctrl('n') => self.open_new_file(),
                Key::Char(c) => {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor(Key::Right);
                }
                Key::Delete => self.document.delete(&self.cursor_position),
                Key::Backspace => {
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        self.move_cursor(Key::Left);
                        self.document.delete(&self.cursor_position);
                    }
                }
                Key::Esc => self.change_mode(EditorMode::View),
                Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::PageUp
                | Key::PageDown
                | Key::End
                | Key::Home => self.move_cursor(pressed_key),
                _ => (),
            }
        }

        self.scroll();
        Ok(())
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_editor()?;

            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }

            callback(self, key, &result);
        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    fn refresh_editor(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::clear_screen();
        Terminal::cursor_position(&Position::default());

        if self.quit {
            Terminal::clear_screen();
            println!("see you later. \r")
        } else {
            self.draw_tildes();
            self.draw_status_bar();
            self.draw_message_bar();

            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::cursor_show();
        Terminal::flush()
    }

    // Open new file opens a document from a given filename, and then pushes that document into the
    // editor's open_documents vector. If a file with the given filename was not found, open a
    // unnamed document without content.
    fn open_new_file(&mut self) {
        let filename = self.prompt("new filepath: ", |_, _, _| {}).unwrap_or(None);
        let mut final_document = Document::default();
        if filename.is_some() {
            let new_document = Document::open(&filename.unwrap());
            if new_document.is_ok() {
                final_document = new_document.unwrap();
            }
        }

        self.documents.push(final_document);
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("ctrl-q quit | ctrl-s save | ctrl-f search");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("error: could not open file '{}'", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            quit: false,
            terminal: Terminal::default().expect("failed to initialize terminal"),
            cursor_position: Position::default(),
            document: document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            editor_mode: EditorMode::View,
            documents: Vec::new(),
            open_document: 0,
        }
    }

    pub fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    // Draw the informative status bar which displays, some helpful commands, and the open
    // documents.
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mod_indicator = if self.document.is_edited() {
            " (edited)"
        } else {
            ""
        };

        let mut open_document_display = String::new();
        for document in &self.documents {
            match &document.file_name {
                Some(file_name) => open_document_display += &file_name.to_string(),
                None => (),
            }
        }

        let mut file_name = "[no name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }

        let editor_mode = if self.editor_mode == EditorMode::View {
            "view".to_string()
        } else {
            "insert".to_string()
        };

        status = format!(
            "{} | {}{} | open: {}",
            editor_mode, file_name, mod_indicator, open_document_display
        );

        let line_indicator = format!(
            "[{}/{}] [{}]",
            self.cursor_position.y.saturating_add(1),
            self.document.len(),
            self.document.file_type(),
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);

        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("x editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn draw_tildes(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
}

fn end(e: std::io::Error) {
    Terminal::clear_screen();
    panic!(e);
}
