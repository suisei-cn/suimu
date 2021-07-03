use clap::Clap;
use suimu::check_csv;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[derive(Clap)]
#[clap(
version = clap::crate_version ! (),
author = clap::crate_authors ! (),
about = clap::crate_description ! ()
)]
struct Opts {
    #[clap(
        about = "The CSV file to check",
        default_value = "suisei-music.csv",
        index = 1,
        required = true
    )]
    csv_file: String,
}

fn main() {
    // Set default logging level to INFO
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Parse arguments
    let opts: Opts = Opts::parse();
    let csv_file = opts.csv_file;
    info!("CSV file: {}", csv_file);

    let read_file = std::fs::read_to_string(csv_file);

    if read_file.is_err() {
        error!("Cannot open CSV file.");
        std::process::exit(2);
    }

    let read_file = read_file.unwrap();

    match check_csv(&read_file) {
        Ok(arr) => {
            info!("CSV successfully validated. {} entries found.", arr.len());
        }
        Err(err) => {
            error!("CSV validation failed: {}", err);
            std::process::exit(1);
        }
    }
}
