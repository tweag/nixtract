use ::std::sync::{Arc, Mutex};
use rayon::prelude::*;

use error::Result;
use nix::{DerivationDescription, FoundDrv};

pub mod error;
pub mod nix;

fn process(
    collected_paths: &Arc<Mutex<std::collections::HashSet<String>>>,
    flake_ref: &str,
    system: &str,
    attribute_path: &str,
    offline: &bool,
) -> Vec<DerivationDescription> {
    log::debug!("Processing derivation: {:?}", attribute_path);

    // call describe_derivation to get the derivation description
    let description = nix::describe_derivation(flake_ref, system, attribute_path, offline).unwrap();

    // use par_iter to call process on all children of this derivation
    let children: Vec<DerivationDescription> = description
        .build_inputs
        .par_iter()
        .map(|build_input| {
            // check if the build_input has already be processed
            let done = {
                let mut collected_paths = collected_paths.lock().unwrap();
                match &build_input.output_path {
                    Some(output_path) => !collected_paths.insert(output_path.clone()),
                    None => false,
                }
            };

            if done {
                log::debug!(
                    "Skipping already processed derivation: {}",
                    build_input.attribute_path
                );
                Vec::new()
            } else {
                process(
                    &collected_paths,
                    flake_ref,
                    system,
                    &build_input.attribute_path,
                    offline,
                )
            }
        })
        .flatten()
        .collect();

    // combine the children and this derivation into a single Vec
    let mut descriptions = children;
    descriptions.push(description);
    descriptions
}

pub fn nixtract(
    flake_ref: impl AsRef<str>,
    system: impl AsRef<str>,
    attribute_path: Option<impl AsRef<str>>,
    offline: &bool,
) -> Result<Vec<DerivationDescription>> {
    let flake_ref = flake_ref.as_ref();
    let system = system.as_ref();
    let attribute_path = attribute_path.as_ref();

    log::info!(
        "Starting nixtract with flake_ref: {}, system: {}, attribute_path: {:?}",
        flake_ref,
        system,
        attribute_path.map(AsRef::as_ref).unwrap_or("")
    );

    let collected_paths: Arc<Mutex<std::collections::HashSet<String>>> =
        Arc::new(Mutex::new(std::collections::HashSet::new()));

    // call find_attribute_paths to get the initial set of derivations
    let attribute_paths = nix::find_attribute_paths(flake_ref, system, attribute_path, offline)?;

    // Combine all AttributePaths into a single Vec
    let mut derivations: Vec<FoundDrv> = Vec::new();
    for attribute_path in attribute_paths {
        derivations.extend(attribute_path.found_drvs);
    }

    // call process on every foundDrv
    let descriptions: Vec<DerivationDescription> = derivations
        .par_iter()
        .map(|found_drv| {
            process(
                &collected_paths,
                flake_ref,
                system,
                &found_drv.attribute_path,
                offline,
            )
        })
        .flatten()
        .collect();

    Ok(descriptions)
}
