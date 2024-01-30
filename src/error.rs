pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not parse the Nix CLI output: {0}")]
    SerdeJSON(#[from] serde_json::Error),

    #[error("Nix exited with a non-zero exit code: {0:#?}: {1}")]
    NixCommand(Option<i32>, String),

    #[error("IO error when calling Nix: {0}")]
    NixIO(#[from] std::io::Error),
}
