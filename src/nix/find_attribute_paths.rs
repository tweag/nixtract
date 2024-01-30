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
    pub derivation_path: String,
    pub output_path: String,
}

pub fn find_attribute_paths(
    flake_ref: impl AsRef<str>,
    system: Option<impl AsRef<str>>,
    attribute_path: Option<impl AsRef<str>>,
    offline: &bool,
) -> Result<Vec<AttributePaths>> {
    let lib = Lib::new()?;

    let expr = include_str!("find_attribute_paths.nix");

    // Create a scope so env_vars isn't needlessly mutable
    let env_vars: HashMap<String, String> = {
        let mut res =
            HashMap::from([("TARGET_FLAKE_REF".to_owned(), flake_ref.as_ref().to_owned())]);
        if let Some(attribute_path) = attribute_path {
            res.insert(
                "TARGET_ATTRIBUTE_PATH".to_owned(),
                attribute_path.as_ref().to_owned(),
            );
        }
        if let Some(system) = system {
            res.insert("TARGET_SYSTEM".to_owned(), system.as_ref().to_owned());
        }
        res
    };

    // Run the nix command, with the provided environment variables and expression
    let mut command: Command = Command::new("nix");
    command
        .arg("eval")
        .arg("-I")
        .arg(format!("lib={}", lib.path().to_string_lossy()))
        .args(&["--json", "--expr", expr])
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
        if !line.starts_with("trace: ") {
            return Err(Error::NixCommand(output.status.code(), stderr.to_string()));
        } else {
            let attribute_paths: AttributePaths =
                serde_json::from_str(line.trim_start_matches("trace: "))?;
            res.push(attribute_paths);
        }
    }

    Ok(res)
}
