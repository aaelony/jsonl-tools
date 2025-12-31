use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};
use tracing::{Level, error, info, span, warn};

/// Custom error type for HTTP operations
#[derive(Debug)]
pub enum HttpError {
    Io(io::Error),
    Json(serde_json::Error),
    Network(String),
    Other(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Io(e) => write!(f, "IO error: {}", e),
            HttpError::Json(e) => write!(f, "JSON error: {}", e),
            HttpError::Network(e) => write!(f, "Network error: {}", e),
            HttpError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HttpError::Io(e) => Some(e),
            HttpError::Json(e) => Some(e),
            HttpError::Network(_) => None,
            HttpError::Other(_) => None,
        }
    }
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        HttpError::Io(error)
    }
}

impl From<serde_json::Error> for HttpError {
    fn from(error: serde_json::Error) -> Self {
        HttpError::Json(error)
    }
}

/// Trait for different JSONL data backends
pub trait JsonlReader {
    type Error: std::error::Error + Send + Sync + 'static;

    fn load(&mut self) -> Result<(), Self::Error>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> Option<&Value>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Value>;
    fn replace(&mut self, index: usize, value: Value) -> Result<(), Self::Error>;
    fn iter(&self) -> Box<dyn Iterator<Item = &Value> + '_>;
    fn source_name(&self) -> &str;
    fn push(&mut self, value: Value) -> Result<(), Self::Error>;
}

/// File-based JSONL reader (current implementation)
pub struct FileJsonlReader {
    path: PathBuf,
    filename: String,
    data: Vec<Value>,
}

impl FileJsonlReader {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            path,
            filename,
            data: Vec::new(),
        }
    }
}

impl JsonlReader for FileJsonlReader {
    type Error = io::Error;

    fn load(&mut self) -> Result<(), Self::Error> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        self.data.clear();

        for line_result in reader.lines() {
            let line = line_result?;
            if !line.trim().is_empty() {
                let json: Value = serde_json::from_str(&line)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                self.data.push(json);
            }
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get(&self, index: usize) -> Option<&Value> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.data.get_mut(index)
    }

    fn replace(&mut self, index: usize, value: Value) -> Result<(), Self::Error> {
        if index < self.data.len() {
            self.data[index] = value;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Index {} out of bounds", index),
            ))
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Value> + '_> {
        Box::new(self.data.iter())
    }

    fn source_name(&self) -> &str {
        &self.filename
    }

    fn push(&mut self, value: Value) -> Result<(), Self::Error> {
        self.data.push(value);
        Ok(())
    }
}

/// In-memory JSONL reader
pub struct MemoryJsonlReader {
    name: String,
    data: Vec<Value>,
}

impl MemoryJsonlReader {
    pub fn new(name: String, data: Vec<Value>) -> Self {
        Self { name, data }
    }

    pub fn from_strings(name: String, json_lines: Vec<&str>) -> Result<Self, serde_json::Error> {
        let data = json_lines
            .into_iter()
            .map(serde_json::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { name, data })
    }
}

impl JsonlReader for MemoryJsonlReader {
    type Error = io::Error;

    fn load(&mut self) -> Result<(), Self::Error> {
        // Data is already in memory, nothing to load
        Ok(())
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get(&self, index: usize) -> Option<&Value> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.data.get_mut(index)
    }

    fn replace(&mut self, index: usize, value: Value) -> Result<(), Self::Error> {
        if index < self.data.len() {
            self.data[index] = value;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Index {} out of bounds", index),
            ))
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Value> + '_> {
        Box::new(self.data.iter())
    }

    fn source_name(&self) -> &str {
        &self.name
    }

    fn push(&mut self, value: Value) -> Result<(), Self::Error> {
        self.data.push(value);
        Ok(())
    }
}

/// HTTP-based JSONL reader
pub struct HttpJsonlReader {
    url: String,
    data: Vec<Value>,
}

impl HttpJsonlReader {
    pub fn new(url: String) -> Self {
        Self {
            url,
            data: Vec::new(),
        }
    }
}

impl JsonlReader for HttpJsonlReader {
    type Error = HttpError;

    fn load(&mut self) -> Result<(), Self::Error> {
        // Placeholder implementation - in a real scenario you'd use reqwest or similar
        warn!("HTTP reader not fully implemented - this is a placeholder");

        // For now, just return an error indicating it's not implemented
        Err(HttpError::Other(
            "HTTP reader not yet implemented".to_string(),
        ))
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get(&self, index: usize) -> Option<&Value> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.data.get_mut(index)
    }

    fn replace(&mut self, index: usize, value: Value) -> Result<(), Self::Error> {
        if index < self.data.len() {
            self.data[index] = value;
            Ok(())
        } else {
            Err(HttpError::Other(format!("Index {} out of bounds", index)))
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Value> + '_> {
        Box::new(self.data.iter())
    }

    fn source_name(&self) -> &str {
        &self.url
    }

    fn push(&mut self, value: Value) -> Result<(), Self::Error> {
        self.data.push(value);
        Ok(())
    }
}

/// Main JsonlData structure, now generic over the reader backend
pub struct JsonlData<R: JsonlReader> {
    pub reader: R,
    pub keys_seen: Option<HashSet<String>>,
    pub key_freqs: Option<Vec<(String, usize)>>,
    pub rows_with_missing_keys: Option<Vec<usize>>,
}

impl<R: JsonlReader> JsonlData<R> {
    pub fn new(mut reader: R) -> Result<Self, R::Error> {
        let span = span!(
            Level::INFO,
            "JsonlData::new",
            filename = reader.source_name()
        );
        let _enter = span.enter();

        reader.load()?;

        let mut instance = Self {
            reader,
            keys_seen: Some(HashSet::new()),
            key_freqs: Some(Vec::new()),
            rows_with_missing_keys: Some(Vec::new()),
        };

        // Analyze the loaded data
        instance.keys_seen = Some(instance.get_all_keys_seen_across_dataset());
        instance.key_freqs = Some(instance.analyze_json_keys());
        instance.rows_with_missing_keys = Some(instance.identify_rows_with_missing_keys());

        Ok(instance)
    }

    pub fn filename(&self) -> &str {
        self.reader.source_name()
    }

    pub fn len(&self) -> usize {
        self.reader.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reader.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.reader.get(index)
    }

    fn get_top_key_combinations(&self, n: usize) -> Vec<(Vec<String>, usize)> {
        let mut combination_freqs: HashMap<Vec<String>, usize> = HashMap::new();

        for val in self.reader.iter() {
            if let Value::Object(map) = val {
                let mut keys: Vec<String> = map.keys().cloned().collect();
                keys.sort();
                *combination_freqs.entry(keys).or_insert(0) += 1;
            }
        }

        let mut sorted_combinations: Vec<(Vec<String>, usize)> =
            combination_freqs.into_iter().collect();
        sorted_combinations.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        sorted_combinations.into_iter().take(n).collect()
    }

    fn analyze_json_keys(&self) -> Vec<(String, usize)> {
        let mut key_counts: BTreeMap<String, usize> = BTreeMap::new();

        for value in self.reader.iter() {
            self.collect_row_keys(value, &mut key_counts, String::new());
        }

        let mut sorted_keys: Vec<(String, usize)> = key_counts.into_iter().collect();
        sorted_keys.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        sorted_keys
    }

    fn collect_row_keys(
        &self,
        value: &Value,
        key_counts: &mut BTreeMap<String, usize>,
        prefix: String,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    *key_counts.entry(full_key.clone()).or_insert(0) += 1;
                    self.collect_row_keys(val, key_counts, full_key);
                }
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let array_key = if prefix.is_empty() {
                        format!("[{}]", index)
                    } else {
                        format!("{}[{}]", prefix, index)
                    };
                    self.collect_row_keys(val, key_counts, array_key);
                }
            }
            _ => {}
        }
    }

    fn get_keys_in_row(&self, value: &Value) -> HashSet<String> {
        let mut keys = HashSet::new();
        self.collect_keys_from_value(value, &mut keys, String::new());
        keys
    }

    fn collect_keys_from_value(&self, value: &Value, keys: &mut HashSet<String>, prefix: String) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    let full_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    keys.insert(full_key.clone());
                    self.collect_keys_from_value(v, keys, full_key);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let array_key = if prefix.is_empty() {
                        format!("[{}]", i)
                    } else {
                        format!("{}[{}]", prefix, i)
                    };
                    self.collect_keys_from_value(v, keys, array_key);
                }
            }
            _ => {}
        }
    }

    fn identify_rows_with_missing_keys(&self) -> Vec<usize> {
        let mut rows_with_missing_keys = Vec::new();
        let all_keys = self.get_all_keys_seen_across_dataset();

        for (i, v) in self.reader.iter().enumerate() {
            let row_keys = self.get_keys_in_row(v);
            let missing_keys: HashSet<_> = all_keys.difference(&row_keys).collect();
            if !missing_keys.is_empty() {
                rows_with_missing_keys.push(i);
            }
        }
        rows_with_missing_keys
    }

    fn get_all_keys_seen_across_dataset(&self) -> HashSet<String> {
        let key_counts = self.analyze_json_keys();
        key_counts.into_iter().map(|(key, _)| key).collect()
    }

    pub fn show_keys_found_report(&self) {
        let span = span!(
            Level::INFO,
            "show_keys_found_report",
            filename = self.filename()
        );
        let _ = span.enter();
        println!("===============================");
        println!(
            "Found {} unique JSON keys in file {}",
            self.keys_seen.as_ref().map_or(0, |k| k.len()),
            self.filename()
        );
    }

    pub fn show_keys_frequencies_report(&self) {
        let span = span!(
            Level::INFO,
            "show_keys_frequencies_report",
            filename = self.filename()
        );
        let _ = span.enter();

        println!("===============================");

        if let Some(ref key_freqs) = self.key_freqs {
            let max_key_len = key_freqs
                .iter()
                .map(|(k, _)| k.len())
                .max()
                .unwrap_or(20)
                .max(20);

            println!("{:<width$} {:>12}", "Key", "Count", width = max_key_len);
            println!("{}", "-".repeat(max_key_len + 14));
            for (k, freq) in key_freqs.iter() {
                let fmt_freq = freq.to_formatted_string(&Locale::en);
                println!("\t{:<width$} {:>12}", k, fmt_freq, width = max_key_len);
            }
        }
        println!("\n");
        println!("Rows with missing keys: {:?}", self.rows_with_missing_keys);
    }

    pub fn show_top_key_combinations_report(&self, n: usize) {
        let span = span!(
            Level::INFO,
            "show_top_key_combinations_report",
            filename = self.filename()
        );
        println!(
            "Top {} Most Frequent JSON Key combinations in {}",
            n,
            self.filename()
        );
        let combos = self.get_top_key_combinations(n);

        if combos.is_empty() {
            warn!("No JSON key combinations found.");
            return;
        }

        for (i, (keys, count)) in combos.iter().enumerate() {
            let keys_str = format!("({})", keys.join(", "));
            let formatted_count = count.to_formatted_string(&Locale::en);
            println!(
                "{}. {} - {} occurrence{}",
                i + 1,
                keys_str,
                formatted_count,
                if *count == 1 { "" } else { "s" }
            );
        }
    }

    pub fn show_record(&self, record_id: usize) {
        let span = span!(Level::INFO, "show_record", filename = self.filename());
        let _ = span.enter();

        if let Some(row) = self.reader.get(record_id) {
            println!(
                "Analysis of Record {}: {}",
                record_id,
                serde_json::to_string_pretty(row)
                    .unwrap_or_else(|_| "Invalid JSON row.".to_string())
            );

            let all_keys = self.get_all_keys_seen_across_dataset();
            let row_keys = self.get_keys_in_row(row);
            let missing_keys: Vec<_> = all_keys.difference(&row_keys).collect();

            if !missing_keys.is_empty() {
                warn!("Missing keys in this record: {:?}", missing_keys);
            } else {
                println!("This record contains all keys found in the dataset.");
            }
        } else {
            error!("Record {} not found", record_id);
        }
    }

    pub fn replace_record(&mut self, record_id: usize, new_json: Value) -> Result<(), R::Error> {
        self.reader.replace(record_id, new_json)?;

        // Recompute analysis after replacement
        self.keys_seen = Some(self.get_all_keys_seen_across_dataset());
        self.key_freqs = Some(self.analyze_json_keys());
        self.rows_with_missing_keys = Some(self.identify_rows_with_missing_keys());

        Ok(())
    }
}

// Type aliases for convenience
pub type FileJsonlData = JsonlData<FileJsonlReader>;
pub type MemoryJsonlData = JsonlData<MemoryJsonlReader>;
pub type HttpJsonlData = JsonlData<HttpJsonlReader>;
