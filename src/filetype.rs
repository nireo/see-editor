pub struct FileType {
    name: String,
    highlight_opts: HighlightOptions,
}

#[derive(Default, Copy, Clone)]
pub struct HighlightOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            highlight_opts: HighlightOptions::default(),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn highlight_options(&self) -> HighlightOptions {
        self.highlight_opts
    }

    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("rust"),
                highlight_opts: HighlightOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                },
            };
        }

        Self::default()
    }
}

impl HighlightOptions {
    pub fn numbers(self) -> bool {
        self.numbers
    }

    pub fn strings(self) -> bool {
        self.strings
    }

    pub fn characters(self) -> bool {
        self.characters
    }

    pub fn comments(self) -> bool {
        self.comments
    }
}
