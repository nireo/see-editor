use std::fs;
use std::io;
use std::path;

// Added a base for a browser
#[derive(Default)]
pub struct Browser {
    main_dir: String,          // The base directory which includes all the `files`.
    files: Vec<path::PathBuf>, // A list of path buffers in the main directory
}

impl Browser {
    // update_main_dir takes in a new directory and updates self's main directory and finds all the
    // files in the new directory and updates the files list.
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
