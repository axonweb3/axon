use clap::crate_version;

use common_version::Version;
use core_cli::AxonCli;

fn main() {
    let crate_version = crate_version!();
    let kernel_version = option_env!("AXON_COMMIT_ID")
        .map(|commit_id| Version::new_with_commit_id(crate_version, commit_id))
        .unwrap_or_else(|| Version::new(crate_version));

    if let Err(e) = AxonCli::init(kernel_version.clone(), kernel_version).start() {
        eprintln!("Error {e}");
        std::process::exit(1);
    }
}
