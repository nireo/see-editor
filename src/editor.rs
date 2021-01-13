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

#[derive(PartialEq)]
enum FileMoveDirection {
    Left,  // Move 1->0
    Right, // Move 0->1
}

pub struct Editor {
    quit: bool,                    // A quit signal
    terminal: Terminal,            // Different terminal controls
    cursor_position: Position,     // The coordinates of a cursor
    offset: Position,              // How much the screen is offset from the original view
    status_message: StatusMessage, // The message displayed at the bottom of the screen
    editor_mode: EditorMode,       // The mode the user is in; either View or Insert
    documents: Vec<Document>,      // A list of all the open documents
    document_index: usize,         // A field to keep track of the open document
    previous_key: termion::event::Key,
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

    // Change the editor move to which ever mode in the EditorMode enum.
    fn change_mode(&mut self, to_change: EditorMode) {
        self.editor_mode = to_change;
    }

    // Save the current document. If a the user is editing a unnamed document, this function will
    // prompt them to name that file to save it.
    fn handle_file_save(&mut self) {
        if self.documents[self.document_index].file_name.is_none() {
            let new_name = self.prompt("save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("save stopped".to_string());
                return;
            }
            self.documents[self.document_index].file_name = new_name;
        }

        // Prompt the user with a message describing the execution of the operation.
        if self.documents[self.document_index].save().is_ok() {
            self.status_message = StatusMessage::from("file saved".to_string());
        } else {
            self.status_message = StatusMessage::from("error writing file".to_string());
        }
    }

    // Check if the wants to exit without saving changes. If the user wants to exit without saving,
    // signal a quit request, so that when refreshing the editor the session will end.
    fn check_exit_without_saving(&mut self) {
        // If not edited just don't do anything and quit the program
        if !self.documents[self.document_index].edited() {
            self.quit = true;
            return;
        }

        let action = self
            .prompt("exit without saving? (y/n)", |_, _, _| {})
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

    // Check if the user wants to close the current document without saving.
    fn exit_document_without_save(&mut self, index: usize) -> bool {
        if index > self.documents.len() - 1 {
            // Return false, since in other functions this leads to doing nothing
            return false;
        }

        let action = self
            .prompt("exit current document without saving (y/n))", |_, _, _| {})
            .unwrap_or(None);

        match action {
            Some(action) => {
                if action.to_string() == "yes" || action.to_string() == "y" {
                    return true;
                }
            }
            None => return false,
        }

        false
    }

    // Close current file closes the document window for a certain file. also does checking if that
    // file is changed.
    fn close_current_file(&mut self) {
        // If the documents length is zero, this action is the same as closing the editor.
        if self.documents.len() == 1 {
            self.check_exit_without_saving();
            return;
        }

        if self.document_index == 0 {
            if self.exit_document_without_save(self.document_index) {
                self.documents.remove(self.document_index);
            }
        } else {
            if self.exit_document_without_save(self.document_index) {
                self.document_index -= 1;
                self.documents.remove(self.document_index);
            }
        }
    }

    // Handles different commands from the editor prompt. Similar to the text prompt in vim when
    // typing ':'.
    fn handle_command(&mut self) {
        let command = self.prompt(": ", |_, _, _| {}).unwrap_or(None);

        if command.is_some() {
            // Match the command by the user to some other commands.
            match command.unwrap().as_str() {
                "s" => self.handle_file_save(),
                "sq" => {
                    // Save the file
                    self.handle_file_save();

                    // The function also does not request the user to give any information if the
                    // file is already saved, this is why we first save the file.
                    self.check_exit_without_saving();
                }
                _ => (),
            }
        }
    }

    // Search for a word in the current document. The user can move in the order from left-to-right
    // and vice versa. The user will type the search term in the prompt field and highlight all the
    // matching words.
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;

        // Get the query word.
        let query = self
            .prompt("search: ", |editor, key, query| {
                let mut moved = false;
                match key {
                    // Move from right-to-left
                    Key::Right | Key::Down => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(Key::Right);
                        moved = true;
                    }
                    // Move from left-to-right
                    Key::Left | Key::Up => direction = SearchDirection::Backward,
                    _ => direction = SearchDirection::Forward,
                }
                // If a position is found move the cursor to that position.
                if let Some(position) = editor.documents[editor.document_index].find(
                    &query,
                    &editor.cursor_position,
                    direction,
                ) {
                    editor.cursor_position = position;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(Key::Left);
                }

                // Highlight the position in which the word is.
                editor.documents[editor.document_index].highlight(Some(query));
            })
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }

        self.documents[self.document_index].highlight(None);
    }

    // Handle all the keypresses the user types as input.
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
                Key::Char(':') => self.handle_command(),
                Key::Char('g') => {
                    if self.previous_key == Key::Char('g') {
                        self.move_cursor(Key::End);
                    }
                }
                Key::Ctrl('q') => self.check_exit_without_saving(),
                Key::Ctrl('s') => self.handle_file_save(),
                Key::Ctrl('z') => self.close_current_file(),
                Key::Ctrl('f') => self.search(),
                Key::Ctrl('p') => self.open_new_file(),
                Key::Ctrl('e') => self.move_cursor(Key::End),
                Key::Ctrl('h') => self.move_cursor(Key::Home),
                Key::Left => self.move_in_documents(FileMoveDirection::Left),
                Key::Right => self.move_in_documents(FileMoveDirection::Right),
                _ => (),
            }

            // Store the previous key so we can have keybindings that use more than two keys
            self.previous_key = pressed_key;
        } else if self.editor_mode == EditorMode::Insert {
            // Handle the keypresses in the insert mode, in which the user can edit the document.
            match pressed_key {
                Key::Ctrl('q') => self.check_exit_without_saving(),
                Key::Ctrl('s') => self.handle_file_save(),
                Key::Ctrl('f') => self.search(),
                Key::Ctrl('n') => self.open_new_file(),
                Key::Char(c) => {
                    // Insert the wanted character at the position of the cursor. Also move the
                    // cursor so it seems more interactive.
                    self.documents[self.document_index].insert(&self.cursor_position, c);
                    self.move_cursor(Key::Right);
                }
                Key::Delete => self.documents[self.document_index].delete(&self.cursor_position),
                Key::Backspace => {
                    // Check that we don't use negative indices.
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        // Move the cursor back and remove the character at the cursor position.
                        self.move_cursor(Key::Left);
                        self.documents[self.document_index].delete(&self.cursor_position);
                    }
                }
                // Go into 'view' mode.
                Key::Esc => self.change_mode(EditorMode::View),
                // Explanations for each keybinding found in the `move_cursor` function.
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

    // Prompt the user to type a variable at the bottom of the editor. Also take in a mutable
    // callback function since it helps with making the search feature a lot cleaner, since we want
    // to move the cursor when searching through words.
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
                // Remove one character from the prompt result.
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                // Since the key is enter, we can stop executing and process the result.
                Key::Char('\n') => break,
                // Add a given key to the result prompt.
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                // Stop typing and don't submit, this just makes the lenght of the result 0.
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }

            // Pass the result to callback, only used for searching words.
            callback(self, key, &result);
        }

        // Clear the prompt from the screen.
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    // Refreshes the editor and checks for a quit signal. If a quit signal is found, stop the
    // execution and else draw all the information on the terminal and flush the screen.
    fn refresh_editor(&self) -> Result<(), std::io::Error> {
        // Hide and reset the cursor position and clear the screen.
        Terminal::cursor_hide();
        Terminal::clear_screen();
        Terminal::cursor_position(&Position::default());

        // Check for quit signal
        if self.quit {
            Terminal::clear_screen();
            println!("see you later. \r")
        } else {
            // Draw the rows, status bar and the message bar.
            self.draw_tildes();
            self.draw_status_bar();
            self.draw_message_bar();

            // Update the terminal cursor position
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        // Show the cursor and flush the screen.
        Terminal::cursor_show();
        Terminal::flush()
    }

    // Open new file opens a document from a given filename, and then pushes that document into the
    // editor's open_documents vector. If a file with the given filename was not found, open a
    // unnamed document without content.
    fn open_new_file(&mut self) {
        let filename = self.prompt("new filepath: ", |_, _, _| {}).unwrap_or(None);
        let mut final_document = Document::default();

        // Check that the filename is not invalid
        if filename.is_some() {
            // Check if we can open a new document using the filename, if not use a default new
            // document.
            let new_document = Document::open(&filename.unwrap());
            if new_document.is_ok() {
                final_document = new_document.unwrap();
            }
        }

        self.documents.push(final_document);
    }

    // Move in the list of files by the document index.
    fn move_in_documents(&mut self, direction: FileMoveDirection) {
        if direction == FileMoveDirection::Left && self.document_index > 0 {
            self.document_index -= 1;
        } else if direction == FileMoveDirection::Right
            && self.document_index < self.documents.len() - 1
        {
            self.document_index += 1;
        }
    }

    // Create a default instance of an editor.
    pub fn default() -> Self {
        // Take a filename as an argument to the program.
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("ctrl-q quit | ctrl-s save | ctrl-f search");
        let document = if args.len() > 1 {
            // If the filename is valid open that file in the editor, otherwise open an empty file
            // in the editor.
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("could not open file '{}'", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            quit: false,
            terminal: Terminal::default().expect("failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            editor_mode: EditorMode::View,
            documents: vec![document],
            document_index: 0,
            previous_key: termion::event::Key::Null,
        }
    }

    // Handle the mouse scroll.
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

        // Display a edited message, if the current document is edited without saving.
        let mod_indicator = if self.documents[self.document_index].is_edited() {
            " (edited)"
        } else {
            ""
        };

        // Display all the open files in the editor.
        let mut open_document_display = String::new();
        for document in &self.documents {
            match &document.file_name {
                Some(file_name) => open_document_display += &format!(" {}", &file_name).to_string(),
                None => (),
            }
        }

        // Display the current opened file.
        let mut file_name = "[no name]".to_string();
        if let Some(name) = &self.documents[self.document_index].file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }

        // Display the editor mode
        let editor_mode = if self.editor_mode == EditorMode::View {
            "view".to_string()
        } else {
            "insert".to_string()
        };
        status = format!(
            "{} | {}{} | open: {}",
            editor_mode, file_name, mod_indicator, open_document_display
        );

        // Indicate the current line, max lines and the detected filetype.
        let line_indicator = format!(
            "[{}/{}] [{}]",
            self.cursor_position.y.saturating_add(1),
            self.documents[self.document_index].len(),
            self.documents[self.document_index].file_type(),
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);

        // Shorten the statuc to fit the screen and also set the colors.
        status.truncate(width);
        Terminal::set_bg_color(color::Rgb(255, 255, 255));
        Terminal::set_fg_color(color::Rgb(63, 63, 63));
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    // Draw the message bar.
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    // Draw a welcome message in the middle of the screen.
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("see -- version {}", VERSION);
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
        let height = self.documents[self.document_index].len();
        let mut width = if let Some(row) = self.documents[self.document_index].row(y) {
            row.len()
        } else {
            0
        };

        match key {
            // Move cursor up
            Key::Up => y = y.saturating_sub(1),

            // Move cursor down
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }

            // Move cursor left
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.documents[self.document_index].row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }

            // Move cursor right
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }

            // Move the cursor by the height of the terminal
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }

            // Move the cursor down by the height of the terminal
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,

            // Home the cursor till the end of the road
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.documents[self.document_index].row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    // Draw a single row to the terminal screen.
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);

        println!("{}\r", row)
    }

    // Draw all the rows in a document.
    fn draw_tildes(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) =
                self.documents[self.document_index].row(terminal_row as usize + self.offset.y)
            {
                self.draw_row(row);
            } else if self.documents[self.document_index].is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
}

// End the execution of the screen.
fn end(e: std::io::Error) {
    Terminal::clear_screen();
    panic!(e);
}
