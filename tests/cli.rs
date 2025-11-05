use assert_cmd::Command;
use predicates::prelude::*;
use anyhow::Result;

#[test]
fn cli_parses_file_and_prints_tree() -> Result<()> {
    let path = "tests/samples/simple.txt";

    Command::cargo_bin("xml_parser")?
        .args(["parse", path])
        .assert()
        .success()
        .stdout(predicate::str::contains("<root>").and(predicate::str::contains("<item>")));

    Ok(())
}

#[test]
fn cli_gets_single_tag_content() -> Result<()> {
    let path = "tests/samples/simple.txt";

    Command::cargo_bin("xml_parser")?
        .args(["parse", path, "-get", "item"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    Ok(())
}

#[test]
fn cli_handles_unknown_command() -> Result<()> {
    Command::cargo_bin("xml_parser")?
        .arg("strange command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown command"));

    Ok(())
}

#[test]
fn cli_reports_missing_file() -> Result<()> {
    let path = "tests/samples/missing.xml";

    Command::cargo_bin("xml_parser")?
        .args(["parse", path])
        .assert()
        .failure()
        .stderr(predicate::str::contains("The system cannot find the file specified"));

    Ok(())
}
