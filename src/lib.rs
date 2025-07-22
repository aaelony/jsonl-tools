pub mod jsonl;

use jsonl::JsonlData;

use std::{env, path::PathBuf};
use tracing::{Level, error, info, span};
use tracing_subscriber::{self, fmt::format::FmtSpan};

// use tracing::{Level, debug, error, info, span, trace, warn};
// use tracing_subscriber::{self, fmt, fmt::format::FmtSpan, prelude::*};
// event!(Level::INFO, "foo");

fn print_welcome() {
    info!(
        "Welcome to {} (Version {})!",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
}

/// Initializes tracing subscriber for logging.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .init(); // No logging will print prior to this line!
    print_welcome();
}

/// Process the file based on the provided filename.
pub fn process_file(filename: String) {
    let span = span!(Level::INFO, "process_file", filename = filename);
    let _ = span.enter();
    let mut path = PathBuf::new();
    path.push(filename);

    let data = JsonlData::new(path.clone());
    data.show_keys_found_report();
    data.show_keys_frequencies_report();
    data.show_top_key_combinations_report(5);
    // Example of showing a specific record (optional)
    // let record_id = 10;
    // data.show_record(record_id);
}

/// Parses command-line arguments to get the filename.
/// Returns `Some(String)` if `--filename` is provided, otherwise `None`.
pub fn parse_cli_arguments() -> Option<String> {
    let span = span!(Level::INFO, "parse_cli_arguments");
    let _ = span.enter();
    let mut arg_filename: Option<String> = None;

    for arg in env::args() {
        if let Some(val) = arg.strip_prefix("--filename=") {
            arg_filename = Some(val.to_string());
        }
    }

    arg_filename
}

pub fn run() {
    // Initialize logging
    init_tracing();

    // Parse the filename argument
    match parse_cli_arguments() {
        Some(f) => process_file(f),
        None => error!("No --filename argument provided for JSON Lines parsing."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let filename = "data/test.jsonl".to_string();
        init_tracing();
        process_file(filename);
    }
}
