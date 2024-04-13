use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=.git/HEAD");
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("should be able to get commit hash");
    let git_hash = String::from_utf8(output.stdout).expect("commit hash should be valid utf8");
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
