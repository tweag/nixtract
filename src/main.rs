use clap::Parser;
use nixtract::nixtract;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long = "target-flake-ref")]
    flake_ref: String,
    #[arg(short, long = "target-attribute-path")]
    attribute_path: String,
    #[arg(short, long = "target-system")]
    system: String,
    #[arg(long)]
    offline: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let opts: Args = Args::parse();

    // Initialize the logger with the provided verbosity
    env_logger::Builder::new()
        .filter_level(opts.verbose.log_level_filter())
        .init();

    // Call the nixtract function with the provided arguments
    let results = nixtract(
        opts.flake_ref,
        opts.system,
        opts.attribute_path,
        &opts.offline,
    )
    .unwrap();

    // Display the results
    for result in results {
        println!("{}", serde_json::to_string(&result).unwrap());
    }
}
