# LINZ Public S3 Bucket Query

A command-line tool for searching and processing LINZ S3 assets using STAC (Spatial Data on the Web) specifications. The tool will print asset URLs to stdout and optionally download the resource.
This fills a very specific need that could probably have been resolved using existing solutions, but this works for my purposes.

---

## Features

- Search S3 buckets for tiles based on spatial coordinates.
- Process search results, including counting tile numbers.
- Prompt users to choose a dataset from search results.
- Download tiles or print their URLs.

---

## Requirements

If you want to build the project yourself:

- **Rust 2021 Edition**
- **Cargo Dependencies**:
  - `clap` for command-line argument parsing.
  - `env_logger` for logging configuration.
  - `futures` for async/await support.
  - `log` for logging utilities.
  - `regex` for regular expression matching.
  - `stac` for STAC specification implementation.
  - `tokio` for asynchronous runtime.

---

## Usage

### Pre-Built Binaries

Pre-built binaries are available for **Windows**, **macOS**, and **Linux**. You can download the appropriate binary for your operating system from the [Releases](https://github.com/your-repo-name/releases) page.

1. Download the binary for your operating system.
2. Run the tool from your terminal:

   ```bash
   ./linz_s3 <bucket> <lat> <lon> <lat1> <lon1>
   ```

   or

     ```bash
   ./linz_s3 --help
   ```

   Follow the prompts to search and process S3 assets.

### Build from Source

If you prefer to build the project yourself, you will need rust installed on your system.

1. Clone the repository:

```bash
git clone https://github.com/jrjdavidson/linz_S3/
cd linz_s3
```

Build the project:

```bash
cargo build --release
```

Run the tool:

```bash
./target/release/linz_s3 <bucket> <lat> <lon> <lat1> <lon1>
```

## Notes

- This project uses the STAC specification for spatial data management.
- Error handling and logging are minimal; consider adding more robust error handling and logging mechanisms.

## GitHub Actions

This project uses GitHub Actions to build and test the tool on Windows, macOS, and Linux. Pre-built binaries are automatically generated and uploaded to the Releases page after each new version tag.

## TODO

Add more features, such as processing tile data or exporting results to a file.
Improve error handling and logging for a better user experience.
