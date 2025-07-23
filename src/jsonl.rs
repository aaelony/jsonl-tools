use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};
use tracing::{Level, error, info, span, warn};
// use tracing::{Level, debug, error, info, span, trace, warn};

pub struct JsonlData {
    pub filename: String,
    pub data: Option<Vec<Value>>,
    pub keys_seen: Option<HashSet<String>>,
    pub key_freqs: Option<Vec<(String, usize)>>,
    pub rows_with_missing_keys: Option<Vec<usize>>,
}

impl JsonlData {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let span = span!(Level::INFO, "JsonlData::new", filename = filename.clone());
        let _enter = span.enter();

        let data = match Self::read_json_lines(&path) {
            Ok(d) => Some(d),
            Err(e) => {
                error!("Failed to read JSON lines from file: {}", e);
                None
            }
        };

        let mut instance = Self {
            filename,
            data,
            keys_seen: Some(HashSet::new()),
            key_freqs: Some(Vec::new()),
            rows_with_missing_keys: Some(Vec::new()),
        };

        let keys_seen = if instance.data.is_some() {
            Some(instance.get_all_keys_seen_across_dataset())
        } else {
            Some(HashSet::new())
        };

        let key_freqs = if instance.data.is_some() {
            // Option<Vec<(String, usize)>>
            Some(instance.analyze_json_keys())
        } else {
            let empty_vec: Option<Vec<(String, usize)>> = Some(Vec::new());
            empty_vec
        };

        let rows_with_missing_keys = if instance.data.is_some() {
            Some(instance.identify_rows_with_missing_keys())
        } else {
            Some(Vec::new())
        };

        instance.keys_seen = keys_seen;
        instance.key_freqs = key_freqs;
        instance.rows_with_missing_keys = rows_with_missing_keys;
        instance
    }

    fn read_json_lines(path: &PathBuf) -> io::Result<Vec<Value>> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut json_values = Vec::new();

        for line_result in reader.lines() {
            let line = line_result?;
            if !line.trim().is_empty() {
                let json: Value = serde_json::from_str(&line)?;
                json_values.push(json);
            }
        }

        Ok(json_values)
    }

    fn get_top_key_combinations(&self, n: usize) -> Vec<(Vec<String>, usize)> {
        let mut combination_freqs: HashMap<Vec<String>, usize> = HashMap::new();

        if let Some(ref data) = self.data {
            for val in data {
                if let Value::Object(map) = val {
                    let mut keys: Vec<String> = map.keys().cloned().collect();
                    keys.sort();
                    *combination_freqs.entry(keys).or_insert(0) += 1;
                }
            }
        }

        let mut sorted_combinations: Vec<(Vec<String>, usize)> =
            combination_freqs.into_iter().collect();
        sorted_combinations.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        sorted_combinations.into_iter().take(n).collect()
    }

    fn analyze_json_keys(&self) -> Vec<(String, usize)> {
        let mut key_counts: BTreeMap<String, usize> = BTreeMap::new();

        if let Some(ref data) = self.data {
            for value in data {
                self.collect_row_keys(value, &mut key_counts, String::new());
            }
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
                for (k, _) in map {
                    let _ = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
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
        if let Some(ref data) = self.data {
            let all_keys = self.get_all_keys_seen_across_dataset();

            for (i, v) in data.iter().enumerate() {
                let row_keys = self.get_keys_in_row(v);
                let missing_keys: HashSet<_> = all_keys.difference(&row_keys).collect();
                if !missing_keys.is_empty() {
                    rows_with_missing_keys.push(i);
                }
            }
        }
        rows_with_missing_keys
    }

    fn get_all_keys_seen_across_dataset(&self) -> HashSet<String> {
        let key_counts = self.analyze_json_keys();
        return key_counts.into_iter().map(|(key, _)| key).collect();
    }

    pub fn show_keys_found_report(&self) {
        let span = span!(
            Level::INFO,
            "show_keys_found_report",
            filename = self.filename
        );
        let _ = span.enter();
        info!("===============================");
        info!(
            "Found {} unique JSON keys in file {}",
            self.keys_seen.clone().unwrap().len(),
            self.filename
        );
    }

    pub fn show_keys_frequencies_report(&self) {
        let span = span!(
            Level::INFO,
            "show_keys_frequencies_report",
            filename = self.filename
        );
        let _ = span.enter();

        // info!("Frequencies of JSON Keys seen: {:?}", self.key_freqs);
        info!("===============================");

        if let Some(ref key_freqs) = self.key_freqs {
            let max_key_len = key_freqs
                .iter()
                .map(|(k, _)| k.len())
                .max()
                .unwrap_or(20)
                .max(20);

            info!("{:<width$} {:>12}", "Key", "Count", width = max_key_len);
            info!("{}", "-".repeat(max_key_len + 14));
            for (k, freq) in key_freqs.iter() {
                let fmt_freq = freq.to_formatted_string(&Locale::en);
                info!("\t{:<width$} {:>12}", k, fmt_freq, width = max_key_len);
                // info!("\t{:>8} {:<30}", freq, k)
            }
        }
        info!("\n");
        info!("Rows with missing keys: {:?}", self.rows_with_missing_keys);
    }

    pub fn show_top_key_combinations_report(&self, n: usize) {
        let span = span!(
            Level::INFO,
            "show_top_key_combinations_report",
            filename = self.filename
        );
        info!(
            "Top {} Most Frequent JSON Key combinations in {}",
            n, self.filename
        );
        let combos = self.get_top_key_combinations(n);

        if combos.is_empty() {
            warn!("No JSON key combinations found.");
            return;
        }

        for (i, (keys, count)) in combos.iter().enumerate() {
            let keys_str = format!("({})", keys.join(", "));
            let formatted_count = count.to_formatted_string(&Locale::en);
            info!(
                "{}. {} - {} occurrence{}",
                i + 1,
                keys_str,
                formatted_count,
                if *count == 1 { "" } else { "s" }
            );
        }
    }

    pub fn show_record(&self, record_id: usize) {
        let span = span!(Level::INFO, "show_record", filename = self.filename);
        let _ = span.enter();
        if let Some(ref data) = self.data {
            if let Some(row) = data.get(record_id) {
                info!(
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
                    info!("This record contains all keys found in the dataset.");
                }
            }
        }
    }

    pub fn replace_record(&mut self, record_id: usize, new_json: serde_json::Value) {
        if let Some(vec) = &mut self.data {
            if record_id < vec.len() {
                vec[record_id] = new_json;

                self.keys_seen = Some(self.get_all_keys_seen_across_dataset());
                self.key_freqs = Some(self.analyze_json_keys());
                self.rows_with_missing_keys = Some(self.identify_rows_with_missing_keys());
            } else {
                error!(
                    "Cannot replace record {}, which is out of bounds!",
                    record_id
                );
            }
        } else {
            error!("Self.data is None, cannot replace record!");
        }
    }
}
