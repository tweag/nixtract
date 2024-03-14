use std::{collections::HashMap, process::Command};

use serde::Deserialize;

use super::lib::Lib;
use crate::error::{Error, Result};

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributePaths {
    pub found_drvs: Vec<FoundDrv>,
}

#[derive(Default, Debug, Clone, Deserialize, PartialEq, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FoundDrv {
    pub attribute_path: String,
    /// drv path of the derivation
    /// We discard any values that are not null or String, which occasionally occur (namely false)
    pub derivation_path: Option<String>,
    /// The output path of the derivation
    /// We discard any values that are not null or String, which occasionally occur (namely false)
    pub output_path: Option<String>,
}

pub fn find_attribute_paths(
    flake_ref: &String,
    system: &Option<String>,
    attribute_path: &Option<String>,
    offline: &bool,
    lib: &Lib,
) -> Result<Vec<AttributePaths>> {
    let expr = include_str!("find_attribute_paths.nix");

    // Create a scope so env_vars isn't needlessly mutable
    let env_vars: HashMap<String, String> = {
        let mut res = HashMap::from([
            ("TARGET_FLAKE_REF".to_owned(), flake_ref.to_owned()),
            ("NIXPKGS_ALLOW_UNFREE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_INSECURE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_BROKEN".to_owned(), "1".to_owned()),
        ]);
        if let Some(attribute_path) = attribute_path {
            res.insert("TARGET_ATTRIBUTE_PATH".to_owned(), attribute_path.clone());
        }
        if let Some(system) = system {
            res.insert("TARGET_SYSTEM".to_owned(), system.to_owned());
        }
        res
    };

    // Run the nix command, with the provided environment variables and expression
    let mut command: Command = Command::new("nix");
    command
        .arg("eval")
        .arg("-I")
        .arg(format!("lib={}", lib.path().to_string_lossy()))
        .args(["--json", "--expr", expr])
        .arg("--impure")
        .envs(env_vars);

    if *offline {
        command.arg("--offline");
    }

    let output = command.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check if the nix command was successful
    if !output.status.success() {
        return Err(Error::NixCommand(output.status.code(), stderr.to_string()));
    }

    let mut res: Vec<AttributePaths> = Vec::new();

    for line in stderr.lines() {
        log::info!("find_attribute_paths line: {}", line);

        if !line.starts_with("trace: ") {
            log::warn!(
                "Unexpected output from nix command, attempting to continue: {}",
                line
            );
        } else {
            match serde_json::from_str(line.trim_start_matches("trace: ")) {
                Ok(attribute_paths) => res.push(attribute_paths),
                Err(e) => {
                    log::warn!(
                        "Error parsing found_derivation output: {} {}. Attempting to continue...",
                        attribute_path.clone().unwrap_or_default(),
                        e
                    );
                }
            };
        }
    }

    Ok(res)
}
