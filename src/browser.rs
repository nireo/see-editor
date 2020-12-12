use std::fs;
use std::io;
use std::path;

pub struct Browser {
    main_dir: String,
    files: Vec<path::PathBuf>,
}

impl Browser {
    pub fn default(starting_dir: &str) -> Self {
        Self {
            main_dir: starting_dir.to_string(),
            files: Vec::new(),
        }
    }

    fn update_main_dir(&mut self, new_dir: &str) -> io::Result<()> {
        let mut files = fs::read_dir(new_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.
        files.sort();

        self.main_dir = new_dir.to_string();

        Ok(())
    }
}
