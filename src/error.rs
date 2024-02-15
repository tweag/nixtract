pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not parse the Nix CLI output for attribute path: {0}: {1}")]
    SerdeJSON(String, serde_json::Error),

    #[error("Nix exited with a non-zero exit code: {0:#?}: {1}")]
    NixCommand(Option<i32>, String),

    #[error("IO error when calling Nix: {0}")]
    NixIO(#[from] std::io::Error),

    #[error("Erorr when sending data to a mpsc channel: {0}")]
    Mpsc(Box<std::sync::mpsc::SendError<crate::nix::DerivationDescription>>),
}

// Cannot automatically derive using #[from] because of the Box
impl From<std::sync::mpsc::SendError<crate::nix::DerivationDescription>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::nix::DerivationDescription>) -> Self {
        Error::Mpsc(Box::new(e))
    }
}
