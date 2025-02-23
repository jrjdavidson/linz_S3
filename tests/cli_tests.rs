use assert_cmd::Command;
#[tokio::test]
async fn test_coordinates_search() {
    let lat1 = "-45.9006";
    let lon1 = "170.8860";
    let lat2 = "-45.2865";
    let lon2 = "175.7762";
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg(lat1)
        .arg(lon1)
        .arg("--search-mode")
        .arg("coordinates")
        .arg(lat2)
        .arg(lon2);
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    cmd.assert().success();
}

#[tokio::test]
async fn test_dimensions_search() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("-45.0")
        .arg("167.0")
        .arg("--search-mode")
        .arg("dimensions")
        .arg("1000.0")
        .arg("1000.0");
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    cmd.assert().success();
}

#[tokio::test]
async fn test_invalid_search_mode() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("imagery")
        .arg("40.9006")
        .arg("174.8860")
        .arg("--search-mode")
        .arg("invalid_mode")
        .arg("41.0")
        .arg("175.0");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("invalid value"));
}

#[tokio::test]
async fn test_missing_arguments_for_dimensions() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("-45.0")
        .arg("167.0")
        .arg("--search-mode")
        .arg("dimensions");

    cmd.assert().failure().stderr(predicates::str::contains("Error: Both arg1 (height in meters) and arg2 (width in meters) must be specified for dimension search."));
}
