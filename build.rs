fn main() {
    if let Some(commit_id) = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty", "--exclude", "*"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok())
    {
        println!("cargo:rustc-env=AXON_COMMIT_ID={}", commit_id);
    }
}
