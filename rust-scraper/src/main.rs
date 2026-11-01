use std::{sync::mpsc, thread, time::Instant};

// Import scraping API types.
use rust_scraper::{progress_bar, scrape, Order, ScrapeOpts};

// Application entry point; sets up an async Tokio runtime.
#[tokio::main]
async fn main() {
    // Target archived page to scrape.
    let url = "https://web.archive.org/web/20200518073855/https://www.empireonline.com/movies/features/best-movies-2/";

    let (scraper_sx, scraper_rx) = mpsc::channel::<bool>();
    let (done_sx, done_rx) = mpsc::channel::<bool>();

    let now = Instant::now();

    println!("\nℹ️ URL = {url}");
    println!();

    thread::spawn(move || {
        progress_bar(scraper_rx, done_sx);
    });

    // Run the scraper with descending order and no HTML file saving.
    match scrape!(
        url = String::from(url),
        sx = scraper_sx,
        save_web = false,
        sort = Order::Descending
    )
    .await
    {
        Ok(movies) => {
            let duration = now.elapsed();

            _ = done_rx.recv();
            // prepare print out banner
            let banner = "=".repeat(70);
            println!();
            println!("{banner}");

            // print out movies
            for (rating, movie) in movies.iter().take(10) {
                println!("{rating}.) {movie}")
            }
            println!("{banner}");
            println!();
            println!("ℹ️ Elapsed time is {}ms", duration.as_millis())
        } // Completed successfully.
        Err(e) => eprintln!("{e}"), // Report errors.
    }
}
