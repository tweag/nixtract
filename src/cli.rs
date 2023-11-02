use clap::Parser;

/// A CLI tool to extract the graph of derivations from a Nix flake
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Reference to the flake to extract data from.
    /// Either a reference in the flake registry or a url.
    /// See https://nixos.org/manual/nix/unstable/command-ref/new-cli/nix3-flake#flake-references
    #[arg(long, default_value_t = String::from("nixpkgs"))]
    target_flake_ref: String,

    /// System to extract data for.
    /// It does not have to be your current system, since this is only for evaluation and not
    /// build.
    #[arg(long, default_value_t = String::from("x86_64-linux"))]
    target_system: String,
}

pub fn cli() {
    let args = Args::parse();
    println!("Target flake reference: {:?}", args.target_flake_ref);
    println!("Target system: {:?}", args.target_system);
}
