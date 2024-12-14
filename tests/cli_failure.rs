use std::env;

use assert_cmd::Command;
use rstest::rstest;

use predicates::prelude::predicate;

// Make some snapshot assertions on command output

#[rstest]
#[case::no_cmd_no_flag(vec![], vec![], "Usage: axl [OPTIONS] <COMMAND>")]
#[case::no_cmd_with_flag(vec!["-v"], vec![], "error: 'axl' requires a subcommand but one was not provided")]
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
