use std::process::Command;

fn main() {
    let output = Command::new("git").args(&["describe", "--always", "--dirty"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_VERSION={}", git_hash);
    tauri_build::build()
}
