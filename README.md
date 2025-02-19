# LINZ public S3 bucket query
---
A command-line tool for searching and processing LINZ S3 assets using STAC (Spatial Data on the Web) specifications. Will either dowload assets or print asset urls.
## Features

Search S3 buckets for tiles based on spatial coordinates
Process search results, including counting tile numbers
Prompt user to choose a dataset from search results
## Requirements

- Rust 2021 edition
- Cargo dependencies:
 - clap for command-line argument parsing
 - env_logger for logging configuration
 - futures for async/await support
 - log for logging utilities
 - regex for regular expression matching
 - stac for STAC specification implementation
 - tokio for asynchronous runtime
## Usage

1. Build the project with cargo build
2. Run the tool with cargo run -- <bucket> <lat> <lon> <lat1> <lon1>
3. Follow the prompts to search and process S3 assets
## Notes

- This project uses the STAC specification for spatial data management.
- Error handling and logging are minimal; consider adding more robust error handling and logging mechanisms.
### TODO

- Add more features, such as processing tile data or exporting results to a file
- Improve error handling and logging for a better user experience
