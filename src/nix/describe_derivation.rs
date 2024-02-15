use std::{collections::HashMap, process::Command};

use serde::{Deserialize, Serialize};

use super::lib::Lib;
use crate::error::{Error, Result};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DerivationDescription {
    pub attribute_path: String,
    pub derivation_path: Option<String>,
    pub output_path: String,
    pub outputs: Vec<Output>,
    pub name: String,
    pub parsed_name: ParsedName,
    pub nixpkgs_metadata: NixpkgsMetadata,
    pub src: Option<Source>,
    pub build_inputs: Vec<BuiltInput>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Output {
    pub name: String,
    pub output_path: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ParsedName {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NixpkgsMetadata {
    pub description: String,
    pub pname: String,
    pub version: String,
    pub broken: bool,
    pub homepage: String,
    pub licenses: Option<Vec<License>>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Source {
    pub git_repo_url: String,
    // Revision or tag of the git repo
    pub rev: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct License {
    // Not all licenses in nixpkgs have an associated spdx id
    pub spdx_id: Option<String>,
    pub full_name: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BuiltInput {
    pub attribute_path: String,
    pub build_input_type: String,
    pub output_path: String,
}

pub fn describe_derivation(
    flake_ref: &String,
    system: &Option<String>,
    attribute_path: &String,
    offline: &bool,
    lib: &Lib,
) -> Result<DerivationDescription> {
    let expr = include_str!("describe_derivation.nix");

    // Create a scope so env_vars isn't needlessly mutable
    let env_vars: HashMap<String, String> = {
        let mut res = HashMap::from([
            ("TARGET_FLAKE_REF".to_owned(), flake_ref.to_owned()),
            (
                "TARGET_ATTRIBUTE_PATH".to_owned(),
                attribute_path.to_owned(),
            ),
            ("NIXPKGS_ALLOW_UNFREE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_INSECURE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_BROKEN".to_owned(), "1".to_owned()),
        ]);
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

    // Add --offline if offline is set
    if *offline {
        command.arg("--offline");
    }

    let output = command.output()?;

    // Get stdout, stderr as a String
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    log::debug!("stdout: {}", stdout);

    // Check if the nix command was successful
    if !output.status.success() {
        return Err(Error::NixCommand(output.status.code(), stderr.to_string()));
    }

    // Parse the stdout as JSON
    let description: DerivationDescription = match serde_json::from_str(stdout.trim()) {
        Ok(description) => description,
        Err(e) => return Err(Error::SerdeJSON(attribute_path.to_owned(), e)),
    };

    Ok(description)
}
