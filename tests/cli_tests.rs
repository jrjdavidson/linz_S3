use std::{fs, path::PathBuf, time::SystemTime};

use assert_cmd::Command;
use serial_test::serial;
use tempfile::tempdir;

#[test]
#[serial]
fn test_latlonsearch() {
    let lat1 = "-45.9006";
    let lon1 = "170.8860";
    let lat2 = "-45.2865";
    let lon2 = "175.7762";
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("coordinate")
        .arg(lat1)
        .arg(lon1)
        .arg(lat2)
        .arg(lon2);
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");
    let num_lines = 1; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().success().stdout(pred);
}

#[test]
#[serial]
fn test_areasearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-first-index")
        .arg("area")
        .arg("-45.0")
        .arg("167")
        .arg("1000.0")
        .arg("1000.0");
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    let num_lines = 1; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().success().stdout(pred);
}

#[test]
fn test_invalid_search_mode() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("imagery")
        .arg("invalid_mode")
        .arg("40.9006")
        .arg("174.8860")
        .arg("41.0")
        .arg("175.0");
    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .failure()
        .stdout(pred)
        .stderr(predicates::str::contains("unexpected argument"));
}

#[test]
fn test_missing_arguments_for_areasearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation").arg("area").arg("-45.0").arg("167.0");
    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .failure()
        .stdout(pred)
        .stderr(predicates::str::contains(
            "the following required arguments were not provided:",
        ));
}

#[test]
fn test_missing_arguments_for_coordinatesearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation").arg("coordinate").arg("-45.0");
    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .failure()
        .stdout(pred)
        .stderr(predicates::str::contains(
            "the following required arguments were not provided:",
        ));
}

#[test]
fn test_invalid_latlon_values() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("coordinate")
        .arg("invalid_lat")
        .arg("invalid_lon");
    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .failure()
        .stdout(pred)
        .stderr(predicates::str::contains("error: invalid value"));
}

#[test]
#[serial]
fn test_empty_search_results() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("coordinate")
        .arg("-90.0")
        .arg("-180.0")
        .arg("-90.0")
        .arg("-180.0");

    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .success()
        .stderr(predicates::str::contains("No datasets found"))
        .stdout(pred);
}

#[test]
#[serial]
fn test_all_datasets() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-all")
        // make test more resilient by filtering by name
        .arg("--include-collection-name")
        .arg("New Zealand DEM Hillshade")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("170.8860");

    // Simulate user input for the dataset index
    let num_lines = 2; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().success().stdout(pred);
}

#[test]
fn test_invalid_args() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-first-index")
        .arg("-s")
        .arg("coordinate")
        .arg("-90.0")
        .arg("-180.0")
        .arg("-90.0")
        .arg("-180.0");
    let num_lines = 0; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("error: the argument "))
        .stdout(pred);
}

#[test]
#[serial]
fn test_valid_search_with_download() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762")
        .current_dir(temp_path); // Set the current directory to the temp directory

    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    let num_lines = 1; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().success().stdout(pred);

    // Assert that exactly one file is created in the temporary directory
    let files: Vec<_> = fs::read_dir(temp_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        files.len(),
        1,
        "Expected exactly one file in the temporary directory"
    );
}
#[test]
#[serial]
fn test_valid_search_with_condition() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-first-index")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 1; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert().stdout(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_index() {
    // could improve check
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--include-collection-name")
        .arg("Southland LiDAR 1m")
        .arg("--by-index")
        .arg("1") // Specify the index you want to test
        .arg("coordinate")
        .arg("-45.9006")
        .arg("160.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 300; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert().stdout(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_missing_index() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-index") // No specific index provided, should default to 0
        .arg("45.5")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762");

    cmd.assert()
        .stderr(predicates::str::contains("invalid value"))
        .failure();
}
#[test]
#[serial]
fn test_invalid_search_with_out_of_bounds_index() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--by-index")
        .arg(usize::MAX.to_string()) // Specify an out-of-bounds index
        .arg("coordinate")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let pred = predicates::str::contains("is out of bounds. There are only"); // Adjust the expected error message

    cmd.assert().stderr(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_conditon_and_one_result() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--include-collection-name")
        .arg("Southland LiDAR 1m DEM")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("160.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 300; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert().stdout(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_conditon_and_mulitple_result() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--include-collection-name")
        .arg("Southland")
        .arg("--by-size")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("160.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 300; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert().stdout(pred).success();
}

#[test]
#[serial]
fn test_valid_search_with_multiple_filters() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--include-collection-name")
        .arg("Southland LiDAR 1m DEM (2020-2024)")
        .arg("--include-collection-name")
        .arg("Canterbury LiDAR 1m DSM (2016-2017)")
        .arg("--by-size")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("160.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 2; // Specify the number of lines you want to match
    let pred: predicates::str::RegexPredicate =
        predicates::str::is_match(format!(r"(?m)^(.*Number of Tiles.*\n){{{}}}", num_lines))
            .unwrap();

    cmd.assert().stderr(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_exclusion_filters() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--exclude-collection-name")
        .arg("Hillshade")
        .arg("--by-size")
        .arg("coordinate")
        .arg("-45")
        .arg("167");
    let num_lines = 1; // Specify the number of lines you want to match
    let pred: predicates::str::RegexPredicate =
        predicates::str::is_match(format!(r"(?m)^(.*Number of Tiles.*\n){{{}}}", num_lines))
            .unwrap();
    cmd.assert().stderr(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_exclusion_inclusion_filters() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--disable-download")
        .arg("--include-collection-name")
        .arg("Southland LiDAR 1m DEM (2020-2024)")
        .arg("--include-collection-name")
        .arg("Canterbury LiDAR 1m DSM (2016-2017)")
        .arg("--by-size")
        .arg("--exclude-collection-name")
        .arg("DEM")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("160.8860")
        .arg("-45.2865")
        .arg("175.7762");
    let num_lines = 1; // Specify the number of lines you want to match
    let pred: predicates::str::RegexPredicate =
        predicates::str::is_match(format!(r"(?m)^(.*Number of Tiles.*\n){{{}}}", num_lines))
            .unwrap();
    cmd.assert().stderr(pred).success();
}
#[test]
#[serial]
fn test_valid_search_with_download_and_cache() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    let cache_dir = tempdir().unwrap();
    let cache_path = cache_dir.path();
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--cache")
        .arg(cache_path)
        .arg("--include-collection-name")
        .arg("Southland")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("169.1860")
        .arg("-45.2865")
        .arg("175.7762")
        .current_dir(temp_path); // Set the current directory to the temp directory

    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");
    let num_lines = 2; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().stdout(pred).success();
    let files: Vec<_> = fs::read_dir(cache_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let file_number = 2;
    check_folder_content(&files, file_number);
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--include-collection-name")
        .arg("Southland")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("169.1860")
        .arg("-45.2865")
        .arg("175.7762")
        .current_dir(temp_path); // Set the current directory to the temp directory

    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");
    let num_lines = 2; // Specify the number of lines you want to match
    let pred: predicates::str::RegexPredicate =
        predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();
    cmd.assert().stdout(pred).success();
    let files: Vec<_> = fs::read_dir(temp_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let file_number = 2;
    check_folder_content(&files, file_number);
    // Capture the modification times of all files before the second run
    let mod_times_before: Vec<SystemTime> = files
        .iter()
        .map(|path| fs::metadata(path).unwrap().modified().unwrap())
        .collect();
    // Run the command again
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("--include-collection-name")
        .arg("Southland")
        .arg("coordinate")
        .arg("-45.9006")
        .arg("169.1860")
        .arg("-45.2865")
        .arg("175.7762")
        .current_dir(temp_path); // Set the current directory to the temp directory

    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");
    let num_lines = 2; // Specify the number of lines you want to match
    let pred = predicates::str::is_match(format!(r"^([^\n]*\n){{{}}}$", num_lines)).unwrap();

    cmd.assert()
        .stdout(pred)
        .stderr(predicates::str::contains("files found in cache, 0 files"))
        .success();
    let file_number = 2;
    check_folder_content(&files, file_number);

    // Capture the contents of the cache directory after the second run
    let files: Vec<_> = fs::read_dir(temp_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let file_number = 2;
    check_folder_content(&files, file_number);

    // Capture the modification times of all files after the second run
    let mod_times_after: Vec<SystemTime> = files
        .iter()
        .map(|path| fs::metadata(path).unwrap().modified().unwrap())
        .collect();

    // Compare the modification times before and after the second run
    assert_eq!(mod_times_before, mod_times_after, "Files were overwritten",);
}

fn check_folder_content(files: &[PathBuf], file_number: usize) {
    // Check if there is exactly one subfolder in the temporary directory
    let subfolders: Vec<_> = files.iter().filter(|path| path.is_dir()).collect();
    assert_eq!(
        subfolders.len(),
        1,
        "Expected exactly one subfolder in the temporary directory"
    );

    // Get the path of the subfolder
    let subfolder_path = subfolders[0];

    // Check the contents of the subfolder
    let subfolder_files: Vec<_> = fs::read_dir(subfolder_path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        subfolder_files.len(),
        file_number,
        "Expected exactly one file in the subfolder"
    );
}
