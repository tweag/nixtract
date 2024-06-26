use std::{collections::HashMap, process::Command};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::lib::Lib;
use crate::error::{Error, Result};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
/// All information of a derivation that is extracted directly from nix
pub struct DerivationDescription {
    pub attribute_path: String,
    pub derivation_path: Option<String>,
    pub output_path: Option<String>,
    pub outputs: Vec<Output>,
    pub name: String,
    pub parsed_name: ParsedName,
    pub nixpkgs_metadata: NixpkgsMetadata,
    pub src: Option<Source>,
    pub build_inputs: Vec<BuiltInput>,

    #[serde(skip_deserializing)]
    pub nar_info: Option<super::narinfo::NarInfo>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct Output {
    pub name: String,
    pub output_path: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct ParsedName {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct NixpkgsMetadata {
    pub description: String,
    pub pname: String,
    pub version: String,
    pub broken: bool,
    pub homepage: String,
    pub licenses: Option<Vec<License>>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct Source {
    pub git_repo_url: String,
    // Revision or tag of the git repo
    pub rev: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct License {
    // Not all licenses in nixpkgs have an associated spdx id
    pub spdx_id: Option<String>,
    pub full_name: String,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, JsonSchema)]
pub struct BuiltInput {
    pub attribute_path: String,
    pub build_input_type: String,
    pub output_path: Option<String>,
}

#[derive(Clone)]
pub struct DescribeDerivationArgs<'a> {
    pub flake_ref: &'a String,
    pub system: &'a Option<String>,
    pub attribute_path: String,
    pub offline: bool,
    pub runtime_only: bool,
    pub include_nar_info: bool,
    pub binary_caches: &'a [String],
    pub lib: &'a Lib,
}

impl<'a> From<crate::ProcessingArgs<'a>> for DescribeDerivationArgs<'a> {
    fn from(args: crate::ProcessingArgs<'a>) -> Self {
        DescribeDerivationArgs {
            flake_ref: args.flake_ref,
            system: args.system,
            attribute_path: args.attribute_path,
            offline: args.offline,
            runtime_only: args.runtime_only,
            include_nar_info: args.include_nar_info,
            binary_caches: args.binary_caches,
            lib: args.lib,
        }
    }
}

pub fn describe_derivation(args: &DescribeDerivationArgs) -> Result<DerivationDescription> {
    let expr = include_str!("describe_derivation.nix");

    // Create a scope so env_vars isn't needlessly mutable
    let env_vars: HashMap<String, String> = {
        let mut res = HashMap::from([
            ("TARGET_FLAKE_REF".to_owned(), args.flake_ref.to_owned()),
            (
                "TARGET_ATTRIBUTE_PATH".to_owned(),
                args.attribute_path.to_owned(),
            ),
            ("NIXPKGS_ALLOW_UNFREE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_INSECURE".to_owned(), "1".to_owned()),
            ("NIXPKGS_ALLOW_BROKEN".to_owned(), "1".to_owned()),
            (
                "RUNTIME_ONLY".to_owned(),
                if args.runtime_only { "1" } else { "0" }.to_owned(),
            ),
        ]);
        if let Some(system) = args.system {
            res.insert("TARGET_SYSTEM".to_owned(), system.to_owned());
        }
        res
    };

    // Run the nix command, with the provided environment variables and expression
    let mut command: Command = Command::new("nix");
    command
        .arg("eval")
        .arg("-I")
        .arg(format!("lib={}", args.lib.path().to_string_lossy()))
        .args(["--json", "--expr", expr])
        .arg("--impure")
        .args(["--extra-experimental-features", "flakes nix-command"])
        .envs(env_vars);

    // Add --offline if offline is set
    if args.offline {
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
    let mut description: DerivationDescription = match serde_json::from_str(stdout.trim()) {
        Ok(description) => description,
        Err(e) => return Err(Error::SerdeJSON(args.attribute_path.to_owned(), e)),
    };

    if args.include_nar_info && description.output_path.is_some() {
        let output_path = description.output_path.clone().unwrap();
        let narinfo = super::narinfo::NarInfo::fetch(&output_path, args.binary_caches)?;

        description.nar_info = narinfo;
    };

    Ok(description)
}
