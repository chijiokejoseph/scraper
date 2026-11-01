// Declare the net module containing scraping logic.
mod display;
mod net;

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

// Re-export commonly used types and functions from the scrape_fn module.
pub use display::progress_bar;
pub use net::{scrape, Order, ScrapeOpts};

// Returns the path to the data directory, creating it if it does not already exist.
pub fn get_path() -> io::Result<PathBuf> {
    // Start from the current working directory.
    let mut dir = env::current_dir()?;
    // Append the "data" subdirectory.
    dir.push("data");
    match fs::create_dir_all(&dir) {
        Ok(_) => (), // Directory created or already exists.
        Err(e) => {
            // Print a message on failure to create the directory.
            println!(
                "Could not create directory at path: {}",
                dir.to_str().unwrap()
            );
            // Propagate the error.
            Err(e)?
        }
    };
    Ok(dir) // Return the data directory path.
}

pub fn flush() {
    io::stdout()
        .flush()
        .expect("failed to flush stdout buffer to screen");
}
