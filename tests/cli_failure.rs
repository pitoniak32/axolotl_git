use assert_cmd::Command;
use rstest::rstest;

use predicates::prelude::predicate;

// Make some snapshot assertions on command output

#[rstest]
#[case::no_cmd_no_flag(vec![], vec![], "Usage: axl [OPTIONS] [COMMAND]")]
#[case::no_cmd_with_flag(vec!["-v"], vec![], "No command was provided! To see commands use `--help`.")]
#[case::project_cmd_no_sub_cmd(vec!["project"], vec![], "Usage: axl project [OPTIONS] <COMMAND>")]
#[case::project_cmd_open_sub_cmd(vec!["project", "open"], vec![("PROJECTS_DIRECTORY_FILE", "/test/file/path.yml")], "Usage: axl project open --projects-directory-file <PROJECTS_DIRECTORY_FILE> --multiplexer <MULTIPLEXER>")]
fn axl_project_no_args(
    #[case] args: Vec<&str>,
    #[case] envs: Vec<(&str, &str)>,
    #[case] expected_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let mut cmd = Command::cargo_bin("axl")?;

    for (key, value) in envs {
        cmd.env(key, value);
    }

    for arg in args {
        cmd.arg(arg);
    }

    // Act / Assert
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(expected_msg));

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
