use anyhow::Result;
use assert_cmd::Command;

fn main() -> Result<()> {
    let o = Command::cargo_bin("../axl")?
        .env(
            "AXL_PROJECTS_CONFIG_PATH",
            "examples/files/project_config.yml",
        )
        .arg("project")
        .arg("list")
        .ok()?;

    println!(
        "Full JSON Project Objects:\n{}",
        String::from_utf8(o.stdout)?
    );

    let o = Command::cargo_bin("../axl")?
        .env(
            "AXL_PROJECTS_CONFIG_PATH",
            "examples/files/project_config.yml",
        )
        .arg("project")
        .arg("list")
        .arg("--tags=clis")
        .arg("--name-only")
        .ok()?;

    println!(
        "Filtered Name Only Projects:\n{}",
        String::from_utf8(o.stdout)?
    );

    Ok(())
}
