//! The main entry point for the nixtract command line tool.
//!
//! Calling this tool starts a subprocess that list top-level derivations (outputPath + attribute path) to its stderr pipe, see `src/nix/find_attribute_paths.nix`.
//! This pipe is consumed in a thread that reads each line and populates a vector.
//! This vector is consumed by rayon threads that will call the `process` function.
//! This function will call a subprocess that describes the derivation (name, version, license, dependencies, ...), see `src/nix/describe_derivation.nix`.
//! When describing a derivation, if dependencies are found and have not been already queued for processing, they are added to the current thread's queue, which makes us explore the entire depth of the graph.
//! Rayon ensures that a thread without work in its queue will steal work from another thread, so we can explore the graph in parallel.
//!
//! The whole system stops once
//! - all top-level attribute paths have been found
//! - all derivations from that search have been processed
//! - all dependencies have been processed
//!
//! Glossary:
//! - output path: full path of the realization of the derivation in the Nix store.
//!     e.g. /nix/store/py9jjqsgsya5b9cpps64gchaj8lq2h5i-python3.10-versioneer-0.28
//! - attribute path: path from the root attribute set to get the desired value.
//!     e.g. python3Derivations.versioneer
use std::{error::Error, io::Write};

use clap::Parser;
use nixtract::{message::Message, nixtract};

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

    /// Attempt to fetch nar info from the binary cache
    #[arg(short = 'n', long, default_value_t = false)]
    include_nar_info: bool,

    /// List of caches to attempt to fetch narinfo from, defaults to the substituters from nix.conf and the `extra-substituters` from provided flake.
    #[arg(short, long)]
    binary_caches: Option<Vec<String>>,

    /// Count of workers to spawn to describe derivations
    #[arg(long)]
    n_workers: Option<usize>,

    /// Pretty print the output
    #[arg(long, default_value_t = false)]
    pretty: bool,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Output the json schema
    #[arg(long, default_value_t = false)]
    output_schema: bool,

    /// Write the output to a file instead of stdout or explicitly use `-` for stdout
    #[arg()]
    output_path: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Args = Args::parse();

    // Create the out writer
    let (mut out_writer, to_file) = match opts.output_path.as_deref() {
        None | Some("-") => (
            Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
            false,
        ),
        Some(path) => {
            let file = std::fs::File::create(path)?;
            (Box::new(file) as Box<dyn std::io::Write>, true)
        }
    };

    // If schema is requested, print the schema and return
    if opts.output_schema {
        let schema = schemars::schema_for!(nixtract::DerivationDescription);
        let schema_string = serde_json::to_string_pretty(&schema)?;
        out_writer.write_all(schema_string.as_bytes())?;
        out_writer.write_all(b"\n")?;
        Ok(())
    } else {
        main_with_args(opts, out_writer, to_file)
    }
}

fn main_with_args(
    opts: Args,
    mut out_writer: impl Write,
    to_file: bool,
) -> Result<(), Box<dyn Error>> {
    // Initialize the rayon thread pool with the provided number of workers
    // or use the default number of workers if none is provided
    if let Some(n_workers) = opts.n_workers {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n_workers)
            .build_global()?;
    }

    let (status_tx, status_rx): (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ) = std::sync::mpsc::channel();

    // Initialize the logger if not writing to a file, otherwise we defer it to after we created the MultiProcess
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter_level(opts.verbose.log_level_filter());
    if !to_file {
        // Initialize the logger with the provided verbosity
        let _ = log_builder.try_init();
    }

    // If we are outputing to a file and not stdout, start a gui thread that uses indicatif to display progress
    // If we would always start a MultiProgress progress bar, the output would be mangled by the output we write to stdout ourselves.
    let handle = if to_file {
        let spinner_style =
            indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")?
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let multi = indicatif::MultiProgress::new();
        let logger = log_builder.build();
        let _ = indicatif_log_bridge::LogWrapper::new(multi.clone(), logger).try_init();

        Some(std::thread::spawn(move || {
            // Create a progress bar for rayon thread in the global thread pool
            let mut progress_bars = Vec::new();
            for _ in 0..rayon::current_num_threads() {
                let pb = multi.add(indicatif::ProgressBar::new(0));
                pb.set_style(spinner_style.clone());
                progress_bars.push(pb);
            }

            for message in status_rx {
                match message.status {
                    nixtract::message::Status::Started => {
                        progress_bars[message.id]
                            .set_message(format!("Processing {}", message.path));
                    }
                    nixtract::message::Status::Completed => {
                        progress_bars[message.id]
                            .set_message(format!("Processed {}", message.path));
                        progress_bars[message.id].inc(1);
                    }
                    nixtract::message::Status::Skipped => {
                        progress_bars[message.id].set_message(format!("Skipped {}", message.path));
                    }
                }
            }

            for pb in progress_bars {
                pb.finish();
            }

            multi.clear().expect("Failed to clear the progress bar");
        }))
    } else {
        None
    };

    let results = nixtract(
        opts.flake_ref,
        opts.system,
        opts.attribute_path,
        opts.offline,
        opts.include_nar_info,
        opts.binary_caches,
        Some(status_tx),
    )?;

    // Print the results to the provided output, and pretty print if specified
    for result in results {
        let output = if opts.pretty {
            serde_json::to_string_pretty(&result)?
        } else {
            serde_json::to_string(&result)?
        };

        out_writer.write_all(output.as_bytes())?;
        out_writer.write_all(b"\n")?;
    }

    if let Some(handle) = handle {
        handle.join().expect("Failed to join the gui thread");
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
                    output_schema: bool::default(),
                    // Write output to /dev/null to avoid cluttering the test output
                    output_path: Some("/dev/null".to_string()),
                    include_nar_info: false,
                    binary_caches: None,
                };

                log::info!("Running test for {:?}", path);

                // Set out_writer to /dev/null to avoid cluttering the test output
                let out_writer = std::fs::File::create("/dev/null").unwrap();

                let res = main_with_args(opts, out_writer, true);

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
