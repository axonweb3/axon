fn main() {
    let desc = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty", "--exclude", "*"])
        .output()
        .ok()
        .and_then(|r| String::from_utf8(r.stdout).ok())
        .map(|d| format!(" {d}"))
        .unwrap_or_default();
    println!("cargo:rustc-env=AXON_GIT_DESCRIPTION={desc}");
}
