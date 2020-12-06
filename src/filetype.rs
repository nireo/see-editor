pub struct FileType {
    name: String,
    highlight_opts: HighlightOptions,
}

#[derive(Default)]
pub struct HighlightOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
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

    pub fn highlight_options(&self) -> &HighlightOptions {
        &self.highlight_opts
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
                    primary_keywords: vec![
                        "as".to_string(),
                        "break".to_string(),
                        "const".to_string(),
                        "continue".to_string(),
                        "crate".to_string(),
                        "else".to_string(),
                        "enum".to_string(),
                        "extern".to_string(),
                        "false".to_string(),
                        "fn".to_string(),
                        "for".to_string(),
                        "if".to_string(),
                        "impl".to_string(),
                        "in".to_string(),
                        "let".to_string(),
                        "loop".to_string(),
                        "match".to_string(),
                        "mod".to_string(),
                        "move".to_string(),
                        "mut".to_string(),
                        "pub".to_string(),
                        "ref".to_string(),
                        "return".to_string(),
                        "self".to_string(),
                        "Self".to_string(),
                        "static".to_string(),
                        "struct".to_string(),
                        "super".to_string(),
                        "trait".to_string(),
                        "true".to_string(),
                        "type".to_string(),
                        "unsafe".to_string(),
                        "use".to_string(),
                        "where".to_string(),
                        "while".to_string(),
                        "dyn".to_string(),
                        "abstract".to_string(),
                        "become".to_string(),
                        "box".to_string(),
                        "do".to_string(),
                        "final".to_string(),
                        "macro".to_string(),
                        "override".to_string(),
                        "priv".to_string(),
                        "typeof".to_string(),
                        "unsized".to_string(),
                        "virtual".to_string(),
                        "yield".to_string(),
                        "async".to_string(),
                        "await".to_string(),
                        "try".to_string(),
                    ],
                    secondary_keywords: vec![
                        "bool".to_string(),
                        "char".to_string(),
                        "i8".to_string(),
                        "i16".to_string(),
                        "i32".to_string(),
                        "i64".to_string(),
                        "isize".to_string(),
                        "u8".to_string(),
                        "u16".to_string(),
                        "u32".to_string(),
                        "u64".to_string(),
                        "usize".to_string(),
                        "f32".to_string(),
                        "f64".to_string(),
                    ],
                },
            };
        }

        if file_name.ends_with(".py") {
            return Self {
                name: String::from("python3"),
                highlight_opts: HighlightOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    primary_keywords: vec![
                        "class".to_string(),
                        "def".to_string(),
                        "else".to_string(),
                        "for".to_string(),
                        "if".to_string(),
                        "global".to_string(),
                        "while".to_string(),
                        "return".to_string(),
                        "pass".to_string(),
                        "import".to_string(),
                        "try".to_string(),
                        "except".to_string(),
                        "finally".to_string(),
                        "async".to_string(),
                        "await".to_string(),
                        "elif".to_string(),
                        "raise".to_string(),
                        "with".to_string(),
                    ],
                    secondary_keywords: vec![
                        "True".to_string(),
                        "False".to_string(),
                        "None".to_string(),
                        "and".to_string(),
                        "as".to_string(),
                        "assert".to_string(),
                        "break".to_string(),
                        "continue".to_string(),
                        "del".to_string(),
                        "from".to_string(),
                        "in".to_string(),
                        "is".to_string(),
                        "lambda".to_string(),
                        "nonlocal".to_string(),
                        "not".to_string(),
                        "or".to_string(),
                        "yield".to_string(),
                    ],
                },
            };
        }

        if file_name.ends_with(".go") {
            return Self {
                name: String::from("golang"),
                highlight_opts: HighlightOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    primary_keywords: vec![
                        "break".to_string(),
                        "default".to_string(),
                        "func".to_string(),
                        "interface".to_string(),
                        "select".to_string(),
                        "case".to_string(),
                        "defer".to_string(),
                        "go".to_string(),
                        "map".to_string(),
                        "struct".to_string(),
                        "chan".to_string(),
                        "else".to_string(),
                        "goto".to_string(),
                        "package".to_string(),
                        "switch".to_string(),
                        "const".to_string(),
                        "fallthrough".to_string(),
                        "if".to_string(),
                        "range".to_string(),
                        "type".to_string(),
                        "continue".to_string(),
                        "for".to_string(),
                        "import".to_string(),
                        "return".to_string(),
                        "var".to_string(),
                    ],
                    secondary_keywords: vec![
                        "bool".to_string(),
                        "string".to_string(),
                        "int".to_string(),
                        "int8".to_string(),
                        "int16".to_string(),
                        "int32".to_string(),
                        "int64".to_string(),
                        "uint".to_string(),
                        "uint8".to_string(),
                        "uint16".to_string(),
                        "uint32".to_string(),
                        "uint64".to_string(),
                        "uintptr".to_string(),
                        "byte".to_string(),
                        "rune".to_string(),
                        "float32".to_string(),
                        "float64".to_string(),
                        "complex64".to_string(),
                        "complex128".to_string(),
                    ],
                },
            };
        }

        Self::default()
    }
}

impl HighlightOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }

    pub fn strings(&self) -> bool {
        self.strings
    }

    pub fn characters(&self) -> bool {
        self.characters
    }

    pub fn comments(&self) -> bool {
        self.comments
    }

    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.primary_keywords
    }

    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.secondary_keywords
    }
}
