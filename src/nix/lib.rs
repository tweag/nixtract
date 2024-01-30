//! Nix only accepts a file as included files, so we need to create a temporary file to pass to it
//! TODO: Allow sharing of a single lib instance so we don't copy the file so many times

use crate::error::Result;
use std::io::Write;
use tempfile::NamedTempFile;

pub struct Lib {
    inner: std::path::PathBuf,
}

impl Lib {
    pub fn new() -> Result<Self> {
        let mut file = NamedTempFile::new()?;

        let lib = include_str!("lib.nix");

        write!(file, "{}", lib)?;

        let inner = file.into_temp_path().keep().unwrap();

        Ok(Lib { inner })
    }

    pub fn path(&self) -> &std::path::Path {
        &self.inner
    }
}
