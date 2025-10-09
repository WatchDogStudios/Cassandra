use std::process::Command;

fn main() {
    let ts = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", ts);

    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_SHA={}", git);
        }
    }
    if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
    {
        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_TAG={}", tag);
        }
    }
}
