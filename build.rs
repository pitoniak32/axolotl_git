use std::{fs, path::PathBuf, process::Command};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CargoVcsInfo {
    git: GitVcsInfo,
}

#[derive(Debug, Deserialize)]
struct GitVcsInfo {
    sha1: String,
}

const GIT_SHA_SHORT_MIN: usize = 7;

fn main() {
    println!("cargo::rerun-if-changed=.git/HEAD");

    let git_sha_long = if std::env::var("CARGO_PUBLISH_CI").ok().is_some() {
        // not compatible with `cargo package` or `cargo publish` using `--allow-dirty` flag.
        // the `.cargo_vcs_info.json` file is not written
        serde_json::from_str::<CargoVcsInfo>(
            &fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".cargo_vcs_info.json"),
            )
            .expect("should be able to read cargo_vcs_info.json"),
        )
        .expect("cargo_vcs_info.json should contain expected info")
        .git
        .sha1
        .trim()
        .to_string()
    } else {
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .expect("should be able to get commit hash")
                .stdout,
        )
        .expect("commit hash should be valid utf8")
        .trim()
        .to_string()
    };

    println!("cargo:rustc-env=GIT_SHA_LONG={}", &git_sha_long,);
    println!(
        "cargo:rustc-env=GIT_SHA_SHORT={}",
        &git_sha_long[..GIT_SHA_SHORT_MIN],
    );
}
