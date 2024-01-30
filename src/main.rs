use clap::Parser;
use nixtract::nixtract;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    flake_ref: String,
    #[arg(long)]
    attribute_path: String,
    #[arg(long)]
    system: String,
}

fn main() {
    let opts: Args = Args::parse();

    // Call the nixtract function with the provided arguments
    let results = nixtract(opts.flake_ref, opts.system, opts.attribute_path).unwrap();

    // Convert results into a HashSet
    //let results: HashSet<_> = results.into_iter().map(|desc| desc.output_path).collect();

    // Display the results
    for result in results {
        println!("{}", serde_json::to_string(&result).unwrap());
    }
}
