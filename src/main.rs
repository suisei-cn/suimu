use anyhow::Result;

use structopt::clap;
use structopt::StructOpt;

use suimu::command::*;

#[derive(StructOpt)]
#[structopt(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = clap::crate_description ! ()
)]
enum Suimu {
    Build(BuildOpt),
    BuildInteractive,
    Check(CheckOpt),
}

fn main() -> Result<()> {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts = Suimu::from_args();
    match opts {
        Suimu::Build(build_opt) => build(build_opt)?,
        Suimu::Check(check_opt) => check(check_opt)?,
        Suimu::BuildInteractive => build_interactive()?,
    }
    Ok(())
}