use jsonl_tools::jsonl::JsonlData;

use std::{env, path::PathBuf};
use tracing::{Level, debug, error, info, span, trace, warn};
use tracing_subscriber::{self, fmt, fmt::format::FmtSpan, prelude::*};

// event!(Level::INFO, "foo");

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        // .with_span_events(FmtSpan::FULL)
        .init(); // No logging will pring prior to this line!
    info!(
        "Welcome to {} (Version {})!",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let mut arg_filename: Option<String> = None;

    for arg in std::env::args() {
        if let Some(val) = arg.strip_prefix("--filename=") {
            arg_filename = Some(val.to_string());
        }
    }

    match arg_filename {
        Some(f) => {
            let mut filename = PathBuf::new();
            // filename.push("data");
            // filename.push("conversation_log-20250630-2312.json");

            filename.push(f);
            let data = JsonlData::new(filename.clone());
            data.show_keys_found_report();
            data.show_keys_frequencies_report();

            // let record_id = 10;
            //data.show_record(record_id);
        }
        None => error!("No --filename argument provided for JSON Lines parsing."),
    }
}
