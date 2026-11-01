use std::{
    process,
    sync::mpsc::{self},
    thread,
    time::Duration,
};

use crate::flush;

pub fn progress_bar(scraper_rx: mpsc::Receiver<bool>, done_sx: mpsc::Sender<bool>) {
    // setup variables for controlling progress bar display
    let wait = 60_000;
    let bars = 25;
    let mut ease_time;
    let mut i = 0;

    // set up timeout communcication channel
    let (timeout_sx, timeout_rx) = mpsc::channel::<bool>();

    // spawn a thread that sends a result across a channel was time has elapsed
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(wait as u64));
        let _ = timeout_sx.send(true);
    });

    loop {
        if i == 0 {
            i = 1;
        }

        // setup display banners, progress rate and interval between displays
        let banner = "=".repeat(i);
        ease_time = ((2 * i - 1) * wait) as f64 / (bars as f64).powf(2.0);
        let percent_rate = (i as f64) * 100.0 / (bars as f64);

        // check for result on scraper result channel
        match scraper_rx.try_recv() {
            Ok(_) => {
                // display progress rate at complete stage
                let banner = "=".repeat(bars);
                let percent_rate = 100.0;
                println!("\r✅ Scraping URL [{banner:<bars$}] | {percent_rate:.2}%\n");
                break;
            }
            Err(e) => match e {
                // handle bad state where scraper result channel disconnects early
                mpsc::TryRecvError::Disconnected => {
                    eprintln!("\n❌ Error: scrape result indicator channel closed before sending a message");
                    eprintln!();
                    process::exit(1);
                }
                // if result not yet ready continue
                mpsc::TryRecvError::Empty => {
                    print!("\rℹ️ Scraping URL... [{banner:<bars$}] | {percent_rate:.2}%");
                    flush();
                }
            },
        };

        // check for timeout on its channel
        match timeout_rx.try_recv() {
            Ok(_) => {
                // if the channel has timed out print the matching console message
                println!();
                println!("ℹ️ Scrape operation has timed out...");
                break;
            }
            // handle bad state of timeout channel disconnecting prematurely
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!(
                    "\n❌ Error: timeout result indicator channel closed before sending a message"
                );
                eprintln!();
                process::exit(1);
            }
            // do nothing if result is not ready
            Err(_) => {}
        }

        // increment current bar count
        i += 1;

        // wait for a quadratic ease-in interval between checks and displays
        thread::sleep(Duration::from_millis(ease_time as u64))
    }

    _ = done_sx.send(true);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_progress_bar() {
        // manual test to visually inspect progress bar printout
        let (scraper_sx, scraper_rx) = mpsc::channel::<bool>();
        let (done_sx, _) = mpsc::channel::<bool>();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(60_000));
            _ = scraper_sx.send(true);
        });

        progress_bar(scraper_rx, done_sx);
    }
}
