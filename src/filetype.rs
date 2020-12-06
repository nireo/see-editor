pub struct FileType {
    name: String,
    highlight_opts: HighlightOptions,
}

#[derive(Default)]
pub struct HighlightOptions {
    pub numbers: bool,
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

    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("rust"),
                highlight_opts: HighlightOptions { numbers: true },
            };
        }

        Self::default()
    }
}
