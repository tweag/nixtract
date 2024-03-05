//! This modules parses the get the substituters from the local nix install and the flake ref and combines them into a single list

use crate::error::{Error, Result};

pub type Substituters = Vec<String>;

fn from_flake_ref(flake_ref: &str) -> Result<Substituters> {
    let expr = format!(
        "(import ((builtins.getFlake \"{}\").outPath + \"/flake.nix\")).nixConfig.extra-substituters or []",
        flake_ref
    );

    // Call nix eval on the flake to get the json representation of the nixConfig
    let output = std::process::Command::new("nix")
        .args(["eval", "--json", "--impure"])
        .args(["--expr", &expr])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check if the nix command was successful
    if !output.status.success() {
        return Err(Error::NixCommand(output.status.code(), stderr.to_string()));
    }

    let extra_substituters = match serde_json::from_str(stdout.trim()) {
        Ok(extra_substituters) => extra_substituters,
        Err(e) => return Err(Error::SerdeJSON(flake_ref.to_owned(), e)),
    };

    Ok(extra_substituters)
}

fn from_nix_conf() -> Result<Substituters> {
    let output = std::process::Command::new("nix")
        .args(["show-config", "--json"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check if the nix command was successful
    if !output.status.success() {
        return Err(Error::NixCommand(output.status.code(), stderr.to_string()));
    }

    let config: serde_json::Value = match serde_json::from_str(stdout.trim()) {
        Ok(config) => config,
        Err(e) => return Err(Error::SerdeJSON("nix.conf".to_owned(), e)),
    };

    let substituters = config.get("substituters").and_then(|v| v.get("value"));

    // Parse the substituters array to a Vec<String>
    let substituters = serde_json::from_value(
        substituters
            .unwrap_or(&serde_json::Value::Array(Vec::new()))
            .clone(),
    )
    .map_err(|e| Error::SerdeJSON("nix.conf".to_owned(), e))?;

    Ok(substituters)
}

pub fn get_substituters(flake_ref: String) -> Result<Substituters> {
    let mut substituters = from_nix_conf()?;
    substituters.extend(from_flake_ref(&flake_ref)?);
    Ok(substituters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_flake_ref() {
        let flake_ref = "github:tweag/nixtract";
        let substituters = from_flake_ref(flake_ref);

        assert!(substituters.is_ok());
    }

    #[test]
    fn test_from_nix_conf() {
        let substituters = from_nix_conf();

        assert!(substituters.is_ok());
    }

    /// This test gets the substituters and passes them along to the
    /// NarInfo::fetch function to ensure they are correct.
    #[test]
    fn test_get_substituters() {
        let store_path = "/nix/store/1gxz5nfzfnhyxjdyzi04r86sh61y4i00-hello-2.12.1";
        let substituters = get_substituters("nixpkgs".to_owned()).unwrap();
        let nar_info = crate::narinfo::NarInfo::fetch(store_path, &substituters);

        assert!(nar_info.is_ok_and(|n| n.is_some_and(|n| n.store_path == store_path)))
    }
}
