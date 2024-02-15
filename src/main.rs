use std::error::Error;

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

    /// Run nix evaluation in offline mode
    #[arg(long, default_value_t = false)]
    offline: bool,

    /// Count of workers to spawn to describe derivations
    #[arg(long)]
    n_workers: Option<usize>,

    /// Pretty print the output
    #[arg(long, default_value_t = false)]
    pretty: bool,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Write the output to a file instead of stdout or explicitly use `-` for stdout
    #[arg()]
    output_path: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Args = Args::parse();

    // Initialize the logger with the provided verbosity
    env_logger::Builder::new()
        .filter_level(opts.verbose.log_level_filter())
        .init();

    main_with_args(opts)
}

fn main_with_args(opts: Args) -> Result<(), Box<dyn Error>> {
    // Initialize the rayon thread pool with the provided number of workers
    // or use the default number of workers if none is provided
    if let Some(n_workers) = opts.n_workers {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n_workers)
            .build_global()?;
    }

    // Call the nixtract function with the provided arguments
    let results = nixtract(
        opts.flake_ref,
        opts.system,
        opts.attribute_path,
        opts.offline,
    )?;

    // Create the out writer
    let mut out_writer = match opts.output_path.as_deref() {
        None | Some("-") => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        Some(path) => {
            let file = std::fs::File::create(path)?;
            Box::new(file) as Box<dyn std::io::Write>
        }
    };

    // Print the results
    for result in results {
        let output = if opts.pretty {
            serde_json::to_string_pretty(&result)?
        } else {
            serde_json::to_string(&result)?
        };

        // Append to the out_writer
        out_writer.write_all(output.as_bytes())?;
        out_writer.write_all(b"\n")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_main_fixtures() -> Result<(), Box<dyn Error>> {
        init();

        // For every subdirectory in the tests/fixtures directory
        for entry in fs::read_dir("tests/fixtures").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().canonicalize().unwrap();
            if path.is_dir() {
                // Create the Opts for the main_with_args function
                let opts = Args {
                    flake_ref: path.to_str().unwrap().to_string(),
                    attribute_path: Option::default(),
                    system: Option::default(),
                    offline: bool::default(),
                    n_workers: Option::default(),
                    pretty: bool::default(),
                    verbose: clap_verbosity_flag::Verbosity::default(),
                    // Write output to /dev/null to avoid cluttering the test output
                    output_path: Some("/dev/null".to_string()),
                };

                log::info!("Running test for {:?}", path);

                let res = main_with_args(opts);

                if res.is_ok() {
                    log::info!("Test for {:?} passed", path);
                } else {
                    log::error!("Test for {:?} failed", path);
                    return res;
                }
            }
        }
        Ok(())
    }
}
