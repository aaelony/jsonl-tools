# JSONL Tools

The intention is a utility that can:

 - [X] analyze a [JSONL format](https://jsonlines.org/) file where it is expected that a JSON document of any length occupies 1 row and there are N rows in the file.
 - [X] report on unique JSON keys found
 - [X] report counts of values found per JSON key found
 - [ ] Report number of lines found and the percentage with missing keys.
 - [ ] Identify lines (rows) with missing JSON keys
 - [ ] Provide a way to edit (CRUD) a row by adding needed key/value pairs interactively.


## Example

### Data

```bash
$ more data/test.jsonl 
```

```json
{"id": 1, "name": "Alice", "age": 30, "timestamp": "2025-07-22T12:00:00Z"}
{"id": 2, "name": "Bob", "age": 24, "timestamp": "2025-07-22T12:01:00Z"}
{"id": 3, "name": "Charlie", "age": 28, "timestamp": "2025-07-22T12:02:00Z"}
{"id": 4, "name": "David", "gender":"male", "age": 35, "timestamp": "2025-07-22T12:03:00Z"}
{"id": 5, "name": "Eve", "age": 22, "timestamp": "2025-07-22T12:04:00Z"}
{"id": 6, "name": "Frank", "age": 40, "timestamp": "2025-07-22T12:05:00Z"}
{"id": 7, "name": "Grace", "age": 27, "timestamp": "2025-07-22T12:06:00Z"}
{"id": 8, "name": "Hannah", "age": 33, "timestamp": "2025-07-22T12:07:00Z"}
{"id": 9, "name": "Isaac", "age": 29, "timestamp": "2025-07-22T12:08:00Z"}
{"id": 10, "name": "Jack", "age": 25, "timestamp": "2025-07-22T12:09:00Z"}

```

## Test Example

```bash
 cargo run --release -- --filename=data/test.jsonl 
```

## Example Results

```log

2025-12-03T22:50:12.817000Z  INFO jsonl_tools: Welcome to jsonl_tools (Version 0.1.0)!
2025-12-03T22:50:12.817046Z  INFO parse_cli_arguments: jsonl_tools: enter
2025-12-03T22:50:12.817057Z  INFO parse_cli_arguments: jsonl_tools: exit
2025-12-03T22:50:12.817066Z  INFO jsonl_tools: Processing file: data/test.jsonl
2025-12-03T22:50:12.817075Z  INFO process_file{filename="data/test.jsonl"}: jsonl_tools: enter
2025-12-03T22:50:12.817087Z  INFO process_file{filename="data/test.jsonl"}: jsonl_tools: exit
2025-12-03T22:50:12.817100Z  INFO JsonlData::new{filename="test.jsonl"}: jsonl_tools::jsonl: enter
2025-12-03T22:50:12.817204Z  INFO JsonlData::new{filename="test.jsonl"}: jsonl_tools::jsonl: exit
2025-12-03T22:50:12.817217Z  INFO show_keys_found_report{filename="test.jsonl"}: jsonl_tools::jsonl: enter
2025-12-03T22:50:12.817224Z  INFO show_keys_found_report{filename="test.jsonl"}: jsonl_tools::jsonl: exit
2025-12-03T22:50:12.817230Z  INFO jsonl_tools::jsonl: ===============================
2025-12-03T22:50:12.817235Z  INFO jsonl_tools::jsonl: Found 5 unique JSON keys in file test.jsonl
2025-12-03T22:50:12.817242Z  INFO show_keys_frequencies_report{filename="test.jsonl"}: jsonl_tools::jsonl: enter
2025-12-03T22:50:12.817248Z  INFO show_keys_frequencies_report{filename="test.jsonl"}: jsonl_tools::jsonl: exit
2025-12-03T22:50:12.817254Z  INFO jsonl_tools::jsonl: ===============================
2025-12-03T22:50:12.817263Z  INFO jsonl_tools::jsonl: Key                         Count
2025-12-03T22:50:12.817271Z  INFO jsonl_tools::jsonl: ----------------------------------
2025-12-03T22:50:12.817280Z  INFO jsonl_tools::jsonl: 	age                            10
2025-12-03T22:50:12.817287Z  INFO jsonl_tools::jsonl: 	id                             10
2025-12-03T22:50:12.817294Z  INFO jsonl_tools::jsonl: 	name                           10
2025-12-03T22:50:12.817302Z  INFO jsonl_tools::jsonl: 	timestamp                      10
2025-12-03T22:50:12.817309Z  INFO jsonl_tools::jsonl: 	gender                          1
2025-12-03T22:50:12.817316Z  INFO jsonl_tools::jsonl: 

2025-12-03T22:50:12.817323Z  INFO jsonl_tools::jsonl: Rows with missing keys: Some([0, 1, 2, 4, 5, 6, 7, 8, 9])
2025-12-03T22:50:12.817333Z  INFO jsonl_tools::jsonl: Top 5 Most Frequent JSON Key combinations in test.jsonl
2025-12-03T22:50:12.817352Z  INFO jsonl_tools::jsonl: 1. (age, id, name, timestamp) - 9 occurrences
2025-12-03T22:50:12.817360Z  INFO jsonl_tools::jsonl: 2. (age, gender, id, name, timestamp) - 1 occurrence

```
