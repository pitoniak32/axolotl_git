use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=.git/HEAD");

    let git_hash = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .expect("should be able to get commit hash")
            .stdout,
    )
    .expect("commit hash should be valid utf8")
    .trim()
    .to_string();

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
