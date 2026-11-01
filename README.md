# 🦀🔵🚀 scraper

A side-by-side polyglot web scraping project built to explore data extraction fundamentals and directly compare the syntax, architecture, and developer experience of building concurrent tools in **Go** versus **Rust**.

Both implementations scrape an archived Empire Online "best movies" page, extract each movie's rank and title, sort the results, and save them neatly to `data/movies.txt`. Each tool features a smooth console progress bar and a 60-second safety timeout.

---

## 📂 Repository Structure

```text
scraper/
├── go-scraper/          # 🔵 Go implementation
│   ├── main.go          # CLI entry point and progress display
│   ├── scrapeFn.go      # Scraper logic, options, parsing, and output
│   └── go.mod
├── rust-scraper/        # 🟠 Rust implementation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs      # CLI entry point
│       ├── lib.rs       # Shared helpers and re-exports
│       ├── display.rs   # Progress bar and timeout coordination thread
│       └── net/         # Scraping API and options
└── README.md

```

---

## ⚙️ What It Does

- 🌐 Fetches a historical Wayback Machine snapshot of an Empire Online movie ranking page.
- 🎯 Targets elements matching the CSS selector:
  `🌀 .article-title-description .article-title-description__text .title`
- ✂️ Splits each element's text on `)` to cleanly separate the **rank** and **title**.
- 📊 Sorts the parsed entries according to your preferred order.
- 📝 Writes results directly to `data/movies.txt` (one entry per line formatted as `N) Title`).
- 💾 Optionally saves the fetched raw HTML page to `data/live.html`.
- ⏳ Displays a dynamic console progress bar while running with a 60-second timeout.

---

## 🛠️ Implementations

### 🔵 Go (`go-scraper`)

- **Engine:** Powered by `[github.com/gocolly/colly/v2](https://github.com/gocolly/colly/v2)`.
- **Concurrency:** Runs the scraper inside a goroutine and communicates status safely through channels.
- **API Overview:**
- `Scrape(url string, opts *ScrapeOpts) ([]Movie, error)`
- `Movie{Num int, Title string}`
- `ScrapeOpts{SaveWeb bool, Sort Order}`
- Fluent Builders: `NewScrapeOpts()`, `WithSort(Order)`, `WithSaveWeb(bool)`
- Sort Constants: `Ascending`, `Descending`
  &nbsp;

- **Defaults:** `SaveWeb = false`, `Sort = Ascending`. Prints the first 10 results upon completion.

### 🟠 Rust (`rust-scraper`)

- **Engine:** Built with `reqwest`, `scraper`, `tokio`, `html5ever`, and `soup-kuchiki`.
- **Concurrency:** Async Tokio tasks coordinated with channel messaging and custom timeout threads.
- **API Overview:**
- `scrape(url: String, scraper_sx: mpsc::Sender<bool>, opts: ScrapeOpts) -> Result<Vec<(i32, String)>, Box<dyn Error>>`
- `ScrapeOpts { save_web: bool, sort: Order }`
- `Order::Ascending`, `Order::Descending`
- Custom ergonomic `scrape!` macro supporting keyword arguments.
  &nbsp;

- **Defaults:** `save_web = false`, `sort = Order::Descending`. Prints the first 10 results upon completion.

---

## 📋 Prerequisites

- 🔵 Go 1.27 or newer
- 🟠 Rust stable toolchain with Cargo (and MSVC build tools on Windows)
- 🌍 Active internet connection to fetch the Wayback Machine snapshot

---

## 🚀 Build and Run

### 🔵 Go

```bash
cd go-scraper
go mod download
go run .

```

_Or build a standalone binary:_

```bash
cd go-scraper
go build -o scraper
./scraper

```

### 🟠 Rust

```bash
cd rust-scraper
cargo run

```

_Or build an optimized release binary:_

```bash
cd rust-scraper
cargo build --release
./target/release/rust-scraper

```

---

## ⚙️ Configuration

Neither implementation includes a command-line interface (CLI) parser. Behavior is configured directly in the hard-coded calls within each project's `main` file.

Target URL:

```text
https://web.archive.org/web/20200518073855/https://www.empireonline.com/movies/features/best-movies-2/

```

### 🔵 Go (`main.go`)

```go
movies, err := Scrape(URL, NewScrapeOpts().WithSaveWeb(false).WithSort(Ascending))

```

_Tip: Change `WithSaveWeb(true)` to save `data/live.html`._

### 🟠 Rust (`main.rs`)

```rust
scrape!(
    url = String::from(url),
    sx = scraper_sx,
    save_web = false,
    sort = Order::Descending
)
.await

```

_Tip: Change `save_web = true` to save `data/live.html`._

---

## 📂 Output Files

Both tools automatically create a local `data/` directory in the working directory:

- 📄 `data/movies.txt`: The final sorted movie ranking list.
- 🌐 `data/live.html`: Saved raw HTML page (created only when enabled).
- 🚫 _Note: The `data/` directory is ignored by Git._

---

## ⏱️ Runtime Behavior

When executed, each program follows this lifecycle:

1. 🖨️ Prints the targeted URL to the console.
2. 🔄 Launches the scraper concurrently alongside a smooth terminal progress bar.
3. ⏳ Animates progress using 25 segments mapped across a 60-second quadratic easing window.
4. ✨ Prints the top 10 ranked movies upon a successful scrape.
5. 🛑 Automatically handles timeouts or early disconnections gracefully.

---

## 📦 Dependencies

### 🔵 Go Dependencies

- `[github.com/gocolly/colly/v2](https://github.com/gocolly/colly/v2)` and associated dependency tree.

### 🟠 Rust Dependencies

- `reqwest`, `scraper`, `tokio`, `html5ever`, and `soup-kuchiki`.

---

## 🧪 Testing & Verification

Automated testing for core scraping routes is intentionally minimal:

- 🔵 **Go:** No test suite included.
- 🟠 **Rust:** Includes a visual manual test (`display::tests::test_progress_bar`) for the animation loop.

Both codebases verify cleanly via:

```bash
# Verify Go
cd go-scraper && go build ./...

# Verify Rust
cd rust-scraper && cargo check

```

---

## ⚠️ Known Limitations

- 📌 Target URLs and DOM CSS selectors are hard-coded.
- 🏛️ Relies on a static Wayback Machine snapshot which could experience upstream availability issues.
- 💻 No interactive CLI arguments; configuration options are set directly in `main`.
- 🧹 Parsing logic skips malformed entries, meaning final item counts can occasionally differ from the source page if the layout shifts.
- 💾 File export nuance: Go writes the raw network response body, whereas Rust serializes the parsed DOM document.
