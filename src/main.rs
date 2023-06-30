use core_cli::AxonCli;

fn main() -> anyhow::Result<()> {
    AxonCli::init(
        clap::crate_version!().parse().unwrap(),
        concat!(clap::crate_version!(), env!("AXON_GIT_DESCRIPTION")),
    )
    .start()
}
