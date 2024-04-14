use anyhow::Result;
use assert_cmd::Command;

fn main() -> Result<()> {
    let o = Command::cargo_bin("../axl")?
        .env(
            "AXL_PROJECTS_CONFIG_PATH",
            "examples/files/project_config.yml",
        )
        .arg("project")
        .arg("list-tags")
        .ok()?;

    println!("{}", String::from_utf8(o.stdout)?);

    Ok(())
}
