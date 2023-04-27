use core_cli::AxonCli;

fn main() {
    AxonCli::init(clap::crate_version!()).start();
}
