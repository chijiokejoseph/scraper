// Bring in standard I/O, filesystem and path utilities plus Error trait.
use std::{error::Error, fs, io::Write, path::Path, sync::mpsc};

// HTML serialization utilities from html5ever.
use html5ever::serialize::{serialize, SerializeOpts};
// Scraper crate for CSS selector parsing and HTML document handling.
use scraper::{Html, Selector};
// Soup crate for HTML traversal (currently unused result).
use soup_kuchiki::Soup;

// Import the crate-private helper to locate/create the data directory.
use crate::get_path;

// Options controlling scraping behavior.
#[derive(Debug, Default)]
pub struct ScrapeOpts {
    pub save_web: bool, // Whether to persist the fetched HTML.
    pub sort: Order,    // Sorting order of extracted movies.
}

// Sort order for movie list output.
#[allow(dead_code)]
#[derive(Debug, Default, Copy, Clone)]
pub enum Order {
    #[default]
    Ascending, // Low to high by rank.
    Descending, // High to low by rank.
}

// Writes the parsed HTML document to data/live.html when requested.
fn save_html(data_path: &Path, src_html: &Html) -> Result<(), Box<dyn Error>> {
    // Build path to the output HTML file.
    let html_path = data_path.join("live.html");

    // Open (or create) the file for writing.
    let mut file_opts = fs::OpenOptions::new();
    file_opts.create(true).write(true);
    let mut file = file_opts.open(html_path)?;

    // Truncate any previous content.
    file.set_len(0)?;

    let mut html_output = Vec::new();

    // Serialize the HTML tree into a byte buffer.
    serialize(&mut html_output, src_html, SerializeOpts::default())?;

    // Convert bytes to a UTF-8 string.
    let html_output = String::from_utf8(html_output)?;

    // Write the serialized HTML to disk.
    _ = file.write(html_output.as_bytes())?;
    Ok(())
}

/// # scrape
/// This macros performs scrape operations for different arguments to the scrape
/// function.
///
/// The macros permits scraping with only the url set using the `url=value` construct
/// the parameters `sort` and `save_web` of types `Order` and `bool` respectively are
/// set to their defaults.
///
/// The macros also permits setting either of `sort` and `save_web` or both of them
/// setting the unspecified value, if any, to its default and calling the scrape function
/// with it.
#[macro_export]
macro_rules! scrape {
    // Required positional arguments: url and sx only.
    ($url:expr, $sx:expr) => {
        {
            use super::*;
            scrape($url, $sx, Default::default())
        }
    };
    // Required keyword arguments: url and sx only.
    (url=$url:expr, sx=$sx:expr) => {
        {
            scrape($url, $sx, Default::default())
        }
    };
    // Url and sx plus sort_order positional arguments.
    ($url:expr, $sx:expr, $sort:expr) => {
        {
            let opts = ScrapeOpts{
                sort: $sort,
                ..Default::default()
            };
            scrape($url, $sx, opts)
        }
    };
    // Url and sx plus save_web positional arguments.
    ($url:expr, $sx:expr, $save_web:expr) => {
        {
            let opts = ScrapeOpts{
                save_web: $save_web,
                ..Default::default(),
            };
            scrape($url, $sx, opts)
        }
    };
    (url=$url:expr, sx=$sx:expr, $($key:ident=$val:expr),+ $(,)?) => {
        {
            use std::any::Any;
            // Accumulators for parsed option values.
            let (mut sort, mut save_web) : (Option<Order>, Option<bool>) = (None, None);
            let mut opts: ScrapeOpts = Default::default();
            let mut keys = Vec::new();
            // Ensure no duplicate keyword arguments.
            $(
                let key = stringify!($key);
                if keys.contains(&key) {
                    panic!("'{key}' has already been specified.");
                }
                keys.push(key);
            )+

            $(
                // Box the value as Any to allow runtime type checks.
                let val = Box::new($val) as Box<dyn Any>;
                match stringify!($key) {
                    "sort" => {
                        let sort_val = val.downcast_ref::<Order>();
                        match val {
                            _ if sort_val.is_some() => {
                                let val = sort_val.unwrap();
                                sort = Some(*val);
                            },
                            _ => panic!("The argument to 'sort' must be of type 'Order', got '{:?}'", $val)
                        }
                    },
                    "save_web" => {
                        let save_val = val.downcast_ref::<bool>();
                        match val {
                            _ if save_val.is_some() => {
                                let val = save_val.unwrap();
                                save_web = Some(*val);
                            },
                            _ => panic!("The argument to 'save_web' must be of type 'bool', got {:?}", $val)
                        }
                    }
                    _ => panic!("Only 'sort' and 'save_web' are allowed as keyword arguments")
                }
            )+

            // Apply parsed values if present.
            if let Some(sort_value) = sort {
                opts.sort = sort_value;
            }
            if let Some(value) = save_web {
                opts.save_web = value;
            }
            scrape($url, $sx, opts)
        }
    }
}

pub async fn scrape(
    url: String,
    scraper_sx: mpsc::Sender<bool>,
    opts: ScrapeOpts,
) -> Result<Vec<(i32, String)>, Box<dyn Error>> {
    // Destructure options.
    let ScrapeOpts { sort, save_web } = opts;

    // Fetch the page HTML and ensure HTTP success.
    let response_html = reqwest::get(url).await?.error_for_status()?.text().await?;

    // Parse response into a DOM.
    let html = Html::parse_document(&response_html);

    // Soup wrapper (unused result, kept for potential use).
    let _soup = Soup::new(&response_html);

    // Resolve the data output directory.
    let data_path = match get_path() {
        Ok(dir) => dir,
        Err(e) => {
            println!("Error locating parent directory");
            Err(e)?
        }
    };

    // Optionally persist fetched HTML.
    if save_web {
        save_html(&data_path, &html)?;
    }

    // CSS selector for movie title elements.
    let selector =
        Selector::parse(".article-title-description .article-title-description__text .title")?;

    // Collect matching element text into a vector.
    let matched_movies = html
        .select(&selector)
        .map(|ref el| el.text().collect::<String>())
        .collect::<Vec<_>>();

    // Parsed (rank, title) pairs.
    let mut movies = Vec::new();

    for movie in &matched_movies {
        // Skip entries without a closing paren (no rank).
        if !movie.contains(")") {
            continue;
        }
        // Locate the first ')'.
        let loc = movie.find(")").unwrap();

        // Split into rank part and the rest.
        let (num_str, title) = movie.split_at(loc);

        // Trim and drop the leading ')' from title.
        let title = title.trim().get(1..title.len());
        if title.is_none() {
            continue; // Skip if title slice is invalid.
        }

        let title = title.unwrap();
        let num_str = num_str.trim();

        // Parse the rank as i32.
        let num = num_str.parse::<i32>();

        if num.is_err() {
            continue; // Skip non-numeric ranks.
        }

        let num = num.unwrap();
        movies.push((num, title.to_string())) // Store parsed pair.
    }

    // Sort pairs by rank according to requested order.
    movies.sort_by(|a, b| {
        let (a, b) = (a.0, b.0);
        match sort {
            Order::Ascending => a.cmp(&b),
            Order::Descending => b.cmp(&a),
        }
    });

    // Format results into a single string for persistence.
    let print_out = movies
        .iter()
        .map(|x| format!("{}) {}\n", x.0, x.1))
        .collect::<String>();

    // Resolve output path for movies.txt.
    let movies_path = &data_path.join("movies.txt");

    // write data to output path
    let mut file_opts = fs::OpenOptions::new();
    let file_opts = file_opts.create(true).write(true);
    let mut file = file_opts.open(movies_path)?;

    // Truncate prior contents before writing.
    file.set_len(0)?;

    // Persist formatted list.
    _ = file.write(print_out.as_bytes())?;

    _ = scraper_sx.send(true);
    Ok(movies)
}
