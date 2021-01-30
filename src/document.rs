use crate::FileType;
use crate::Position;
use crate::Row;
use crate::SearchDirection;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    file_type: FileType,
    edited: bool,
}

impl Document {
    // open returns a document based on a filename that is given as a parameter.
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        // Read the content of a file to a string.
        let content = fs::read_to_string(filename)?;

        // Find the correct file type. If the filetype is not found, then we just set a default
        // file type for the document.
        let file_type = FileType::from(filename);

        // Go through the lines in the document.
        let mut rows = Vec::new();
        for value in content.lines() {
            let mut row = Row::from(value);
            row.highlight(&file_type.highlight_options(), None);
            rows.push(row);
        }

        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            file_type,
            edited: false,
        })
    }

    pub fn default(file_name: &str) -> Self {
        Document {
            edited: false,
            file_type: FileType::default(),
            rows: Vec::new(),
            file_name: Some(file_name.to_string()),
        }
    }

    // Returns a reference to row at index
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    // Returns true if the current document is edited, and false if not.
    pub fn edited(&self) -> bool {
        self.edited
    }

    // Return a boolean value about if the document is open or not.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    // Return the amount of rows in a document.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    // Return the document's filetype's name..
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }

    // delete handles the deletion of a character at a given position.
    pub fn delete(&mut self, at: &Position) {
        let len = self.len();

        // We can't remove a character that isn't there.
        if at.y >= len {
            return;
        }

        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
            row.highlight(&self.file_type.highlight_options(), None);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
            row.highlight(&self.file_type.highlight_options(), None);
        }
    }

    // Insert a given char into a given position in a document.
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }

        self.edited = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(&self.file_type.highlight_options(), None);
            self.rows.push(row);
        } else if at.y < self.len() {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
            row.highlight(&self.file_type.highlight_options(), None);
        }
    }

    // Insert newline adds a new line, if the function is used from inside a row the row is
    // splitted from that point.
    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let current_row = &mut self.rows[at.y];
        let mut new_row = current_row.split(at.x);
        current_row.highlight(&self.file_type.highlight_options(), None);
        new_row.highlight(&self.file_type.highlight_options(), None);
        self.rows.insert(at.y + 1, new_row);
    }

    // Save saves all of the changes made to a document into a file.
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = FileType::from(file_name);
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
                row.highlight(&self.file_type.highlight_options(), None);
            }

            self.edited = false;
        }

        Ok(())
    }

    // Find returns a position of an query in a document. The direction dictates if we move up or
    // down in the searches.
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        let mut position = Position { x: at.y, y: at.y };
        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(&self.file_type.highlight_options(), word);
        }
    }

    pub fn is_edited(&self) -> bool {
        self.edited
    }
}
