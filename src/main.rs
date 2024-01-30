use std::path::PathBuf;

use clap::Parser;
use nixtract::nixtract;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long = "target-flake-ref",
        default_value = "nixpkgs",
        help = "The flake URI to extract",
        long_help = "The flake URI to extract, e.g. \"github:tweag/nixtract\""
    )]
    flake_ref: String,

    #[arg(
        short,
        long = "target-attribute-path",
        help = "The attribute path to extract",
        long_help = "The attribute path to extract, e.g. \"haskellPackages.hello\", defaults to all derivations in the flake"
    )]
    attribute_path: Option<String>,

    #[arg(
        short,
        long = "target-system",
        help = "The system to extract",
        long_help = "The system to extract, e.g. \"x86_64-linux\", defaults to the host system"
    )]
    system: Option<String>,
    #[arg(long, default_value_t = false)]
    offline: bool,
    #[arg(long, default_value_t = false)]
    pretty: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[arg()]
    output_path: Option<PathBuf>,
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

    // Print the results
    let output = if opts.pretty {
        serde_json::to_string_pretty(&results).unwrap()
    } else {
        serde_json::to_string(&results).unwrap()
    };

    if let Some(output_path) = opts.output_path {
        log::info!("Writing results to {:?}", output_path);
        std::fs::write(output_path, output).unwrap();
    } else {
        println!("{}", output);
    }
}
