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

    #[error("The provided NarInfo could not be parsed because we encountered an unknown key: {0}")]
    NarInfoUnknownKey(String),

    #[error("The provided value could not be parsed as an integer: {0}")]
    NarInfoParseIntError(#[from] std::num::ParseIntError),

    #[error("The provided NarInfo could not be parsed because we encountered a line without a delimiter: {0}")]
    NarInfoNoDelimiter(String),

    #[error("The provided NarInfo is missing a required field: {0}")]
    NarInfoMissingField(String),

    #[error("The store path is malformed and cannot be used to fetch the narinfo: {0}")]
    NarInfoInvalidPath(String),

    #[error("The narinfo file could not be fetched: {0}")]
    NarInfoReqwest(#[from] reqwest::Error),
}

// Cannot automatically derive using #[from] because of the Box
impl From<std::sync::mpsc::SendError<crate::nix::DerivationDescription>> for Error {
    fn from(e: std::sync::mpsc::SendError<crate::nix::DerivationDescription>) -> Self {
        Error::Mpsc(Box::new(e))
    }
}
