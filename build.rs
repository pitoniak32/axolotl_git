use std::{env::var, fs, path::PathBuf, process::Command};

use serde::Deserialize;

#[derive(Deserialize)]
struct CargoVcsInfo {
    git: GitVcsInfo,
}

#[derive(Deserialize)]
struct GitVcsInfo {
    sha1: String,
}

const GIT_SHA_LEN: usize = 7;

fn main() {
    let git_hash = if var("CARGO_PUBLISH_CI").is_ok_and(|x| x == "true") {
        let vcs_info: CargoVcsInfo = serde_json::from_str(
            &fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".cargo_vcs_info.json"),
            )
            .expect("should be able to read cargo_vcs_info.json"),
        )
        .expect("cargo_vcs_info.json should contain expected info");

        vcs_info.git.sha1[..GIT_SHA_LEN].trim().to_string()
    } else {
        println!("cargo::rerun-if-changed=.git/HEAD");

        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .expect("should be able to get commit hash");
        String::from_utf8(output.stdout)
            .expect("commit hash should be valid utf8")
            .trim()
            .to_string()
    };

    assert!(
        git_hash.len() == GIT_SHA_LEN,
        "git_hash ({git_hash}), must have length = {GIT_SHA_LEN}.\nUpdate `build.rs` to new short hash length if this message is displayed.\nTo determine the new length run: `git rev-parse --short HEAD`."
    );
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
