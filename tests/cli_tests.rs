use assert_cmd::Command;

#[tokio::test]
async fn test_latlonsearch() {
    let lat1 = "-45.9006";
    let lon1 = "170.8860";
    let lat2 = "-45.2865";
    let lon2 = "175.7762";
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate-search")
        .arg(lat1)
        .arg(lon1)
        .arg(lat2)
        .arg(lon2);
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    cmd.assert().success();
}

#[tokio::test]
async fn test_areasearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("area-search")
        .arg("-45.0")
        .arg("167")
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
        .arg("invalid_mode")
        .arg("40.9006")
        .arg("174.8860")
        .arg("41.0")
        .arg("175.0");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("unexpected argument"));
}

#[tokio::test]
async fn test_missing_arguments_for_areasearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("area-search")
        .arg("-45.0")
        .arg("167.0");

    cmd.assert().failure().stderr(predicates::str::contains(
        "the following required arguments were not provided:",
    ));
}

#[tokio::test]
async fn test_missing_arguments_for_coordinatesearch() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation").arg("coordinate-search").arg("-45.0");

    cmd.assert().failure().stderr(predicates::str::contains(
        "the following required arguments were not provided:",
    ));
}

#[tokio::test]
async fn test_invalid_latlon_values() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate-search")
        .arg("invalid_lat")
        .arg("invalid_lon");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("invalid float literal"));
}

#[tokio::test]
async fn test_empty_search_results() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate-search")
        .arg("-90.0")
        .arg("-180.0")
        .arg("-90.0")
        .arg("-180.0");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("No results found"));
}

#[tokio::test]
async fn test_valid_search_with_download() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate-search")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762")
        .arg("--download");
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    cmd.assert().success();
}

#[tokio::test]
async fn test_valid_search_with_condition() {
    let mut cmd = Command::cargo_bin("linz_s3").unwrap();
    cmd.arg("elevation")
        .arg("coordinate-search")
        .arg("-45.9006")
        .arg("170.8860")
        .arg("-45.2865")
        .arg("175.7762")
        .arg("--condition")
        .arg("first");
    // Simulate user input for the dataset index
    cmd.write_stdin("0\n");

    cmd.assert().success();
}
