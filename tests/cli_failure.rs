use rstest::rstest;
use assert_cmd::Command;

use predicates::prelude::predicate;

// Make some snapshot assertions on command output

#[rstest]
#[case::no_args("No command was provided! To see commands use `--help`.")]
fn axl_cmd_help_msgs_success(
    #[case] expected_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let mut cmd = Command::cargo_bin("axl")?;

    // Act / Assert
    cmd.assert().success().stdout(predicate::str::contains(expected_msg));

    Ok(())
}

#[rstest]
#[case::project(vec!["project"], "Usage: axl project [OPTIONS] <COMMAND>")]
#[case::project::open(vec!["project", "open"], "Usage: axl project open --multiplexer <MULTIPLEXER>")]
fn axl_project_no_args(
    #[case] args: Vec<&str>,
    #[case] expected_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let mut cmd = Command::cargo_bin("axl")?;

    for arg in args {
        cmd.arg(arg);
    }

    // Act / Assert
    cmd.assert().failure().stderr(predicate::str::contains(expected_msg));

    Ok(())
}

// #[test]
// fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
//     let file = NamedTempFile::new("sample.txt")?;
//     file.write_str("A test\nActual content\nMore content\nAnother test")?;
//
//     let mut cmd = Command::cargo_bin("grrs")?;
//     cmd.arg("test").arg(file.path());
//     cmd.assert()
//         .success()
//         .stdout(predicate::str::contains("A test\nAnother test"));
//
//     Ok(())
// }
