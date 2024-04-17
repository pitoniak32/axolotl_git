use std::env;

use assert_cmd::Command;
use rstest::rstest;

use predicates::prelude::predicate;

// Make some snapshot assertions on command output

#[rstest]
#[case::no_cmd_no_flag(vec![], vec![], "Usage: axl [OPTIONS] <COMMAND>")]
#[case::no_cmd_with_flag(vec!["-v"], vec![], "error: 'axl' requires a subcommand but one was not provided")]
#[case::project_cmd_no_sub_cmd(vec!["project"], vec![], "Usage: axl project [OPTIONS] <COMMAND>")]
#[case::project_cmd_open_sub_cmd(vec!["project", "open"], vec![("AXL_PROJECTS_CONFIG_PATH", "/test/file/path.yml")], "Usage: axl project open --projects-config-path <PROJECTS_CONFIG_PATH> --multiplexer <MULTIPLEXER>")]
fn axl_project_no_args(
    #[case] args: Vec<&str>,
    #[case] envs: Vec<(&str, &str)>,
    #[case] expected_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    env::remove_var("AXL_DEFAULT_MULTIPLEXER");
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
