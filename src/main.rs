use core_cli::AxonCli;

fn main() -> anyhow::Result<()> {
    AxonCli::init(clap::crate_version!()).start()
}
