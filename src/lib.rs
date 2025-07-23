pub mod jsonl;

use jsonl::{FileJsonlReader, HttpJsonlReader, JsonlData, MemoryJsonlReader};

use std::{env, path::PathBuf};
use tracing::{Level, error, info, span};
use tracing_subscriber::{self, fmt::format::FmtSpan};

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

/// Process the file based on the provided filename using FileJsonlReader.
pub fn process_file(filename: String) {
    let span = span!(Level::INFO, "process_file", filename = filename);
    let _ = span.enter();

    let path = PathBuf::from(filename);
    let reader = FileJsonlReader::new(path);

    match JsonlData::new(reader) {
        Ok(data) => {
            data.show_keys_found_report();
            data.show_keys_frequencies_report();
            data.show_top_key_combinations_report(5);
            // Example of showing a specific record (optional)
            // let record_id = 10;
            // data.show_record(record_id);
        }
        Err(e) => {
            error!("Failed to process file: {}", e);
        }
    }
}

/// Process JSONL data from a URL using HttpJsonlReader.
pub fn process_url(url: String) {
    let span = span!(Level::INFO, "process_url", url = url);
    let _ = span.enter();

    let reader = HttpJsonlReader::new(url);

    match JsonlData::new(reader) {
        Ok(data) => {
            data.show_keys_found_report();
            data.show_keys_frequencies_report();
            data.show_top_key_combinations_report(5);
        }
        Err(e) => {
            error!("Failed to process URL: {}", e);
        }
    }
}

/// Process JSONL data from memory using MemoryJsonlReader.
/// This is useful for testing or when you already have the data in memory.
pub fn process_memory_data(name: String, json_lines: Vec<&str>) {
    let span = span!(Level::INFO, "process_memory_data", name = name);
    let _ = span.enter();

    match MemoryJsonlReader::from_strings(name, json_lines) {
        Ok(reader) => match JsonlData::new(reader) {
            Ok(data) => {
                data.show_keys_found_report();
                data.show_keys_frequencies_report();
                data.show_top_key_combinations_report(5);
            }
            Err(e) => {
                error!("Failed to process memory data: {}", e);
            }
        },
        Err(e) => {
            error!("Failed to parse JSON lines: {}", e);
        }
    }
}

/// Argument parsing that supports different data sources
#[derive(Debug)]
pub enum DataSource {
    File(String),
    Url(String),
    Memory(String, Vec<String>), // name and json lines
}

/// Parses command-line arguments to determine the data source.
/// Returns `Some(DataSource)` if a valid source is provided, otherwise `None`.
pub fn parse_cli_arguments() -> Option<DataSource> {
    let span = span!(Level::INFO, "parse_cli_arguments");
    let _ = span.enter();

    let args: Vec<String> = env::args().collect();

    for arg in &args {
        if let Some(val) = arg.strip_prefix("--filename=") {
            return Some(DataSource::File(val.to_string()));
        }
        if let Some(val) = arg.strip_prefix("--url=") {
            return Some(DataSource::Url(val.to_string()));
        }
    }

    None
}

/// Main entry point that handles different data sources
pub fn run() {
    // Initialize logging
    init_tracing();

    // Parse the arguments to determine data source
    match parse_cli_arguments() {
        Some(DataSource::File(filename)) => {
            info!("Processing file: {}", filename);
            process_file(filename);
        }
        Some(DataSource::Url(url)) => {
            info!("Processing URL: {}", url);
            process_url(url);
        }
        Some(DataSource::Memory(name, lines)) => {
            info!("Processing memory data: {}", name);
            let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
            process_memory_data(name, line_refs);
        }
        None => {
            error!("No valid data source provided. Use --filename=<path> or --url=<url>");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_example() {
        let filename = "data/test.jsonl".to_string();
        init_tracing();
        process_file(filename);
    }

    #[test]
    fn test_memory_reader() {
        init_tracing();

        let json_lines = vec![
            r#"{"name": "Alice", "age": 30, "city": "New York"}"#,
            r#"{"name": "Bob", "age": 25, "occupation": "Engineer"}"#,
            r#"{"name": "Charlie", "age": 35, "city": "San Francisco", "occupation": "Designer"}"#,
        ];

        process_memory_data("test_data".to_string(), json_lines);
    }

    #[test]
    fn test_file_reader_with_nonexistent_file() {
        init_tracing();

        let path = PathBuf::from("nonexistent.jsonl");
        let reader = FileJsonlReader::new(path);

        // This should fail gracefully
        match JsonlData::new(reader) {
            Ok(_) => panic!("Expected error for nonexistent file"),
            Err(e) => {
                info!("Expected error occurred: {}", e);
            }
        }
    }

    #[test]
    fn test_record_replacement() {
        let json_lines = vec![
            r#"{"name": "Alice", "age": 30}"#,
            r#"{"name": "Bob", "age": 25}"#,
        ];

        let reader = MemoryJsonlReader::from_strings("test".to_string(), json_lines).unwrap();
        let mut data = JsonlData::new(reader).unwrap();

        // Replace the first record
        let new_record = json!({"name": "Alice Updated", "age": 31, "city": "Boston"});
        data.replace_record(0, new_record).unwrap();

        // Verify the replacement
        if let Some(record) = data.get(0) {
            assert_eq!(record["name"], "Alice Updated");
            assert_eq!(record["age"], 31);
            assert_eq!(record["city"], "Boston");
        } else {
            panic!("Record 0 should exist");
        }
    }

    #[test]
    fn test_different_backends_same_interface() {
        init_tracing();

        // Test that we can use the same interface for different backends
        let json_lines = vec![
            r#"{"id": 1, "value": "test1"}"#,
            r#"{"id": 2, "value": "test2"}"#,
        ];

        // Memory backend
        let memory_reader =
            MemoryJsonlReader::from_strings("memory_test".to_string(), json_lines).unwrap();
        let memory_data = JsonlData::new(memory_reader).unwrap();

        // Both should work with the same interface
        assert_eq!(memory_data.len(), 2);
        assert!(!memory_data.is_empty());
        assert_eq!(memory_data.filename(), "memory_test");

        // Show that we can call the same methods regardless of backend
        memory_data.show_keys_found_report();
        memory_data.show_keys_frequencies_report();
    }
}
