use core_cli::AxonCli;

fn main() {
    let result = AxonCli::init(
        clap::crate_version!().parse().unwrap(),
        concat!(clap::crate_version!(), env!("AXON_GIT_DESCRIPTION")),
    )
    .start();
    if let Err(e) = result {
        eprintln!("Error {e}");
        std::process::exit(1);
    }
}
