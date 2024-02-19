use ::std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::mpsc;

use error::Result;

mod nix;
pub use nix::*;

pub mod error;

fn process(
    collected_paths: &Arc<Mutex<std::collections::HashSet<String>>>,
    flake_ref: &String,
    system: &Option<String>,
    attribute_path: String,
    offline: bool,
    lib: &nix::lib::Lib,
    // Sender channel to communicate DerivationDescription to the main thread
    tx: mpsc::Sender<DerivationDescription>,
) -> Result<()> {
    log::debug!("Processing derivation: {:?}", attribute_path);

    // call describe_derivation to get the derivation description
    let description = nix::describe_derivation(flake_ref, system, &attribute_path, &offline, lib)?;

    // Send the DerivationDescription to the main thread
    tx.send(description.clone())?;

    // use par_iter to call process on all children of this derivation
    description
        .build_inputs
        .into_par_iter()
        .map(|build_input| -> Result<()> {
            // check if the build_input has already be processed
            let done = {
                let mut collected_paths = collected_paths.lock().unwrap();
                match &build_input.output_path {
                    None => {
                        log::warn!(
                            "Found a derivation without an output_path: {:?}",
                            build_input
                        );
                        false
                    }
                    Some(output_path) => !collected_paths.insert(output_path.clone()),
                }
            };

            if done {
                log::debug!(
                    "Skipping already processed derivation: {}",
                    build_input.attribute_path.to_string()
                );
                return Ok(());
            }

            process(
                collected_paths,
                flake_ref,
                system,
                build_input.attribute_path,
                offline,
                lib,
                tx.clone(),
            )
        })
        .collect::<Result<Vec<()>>>()?;

    Ok(())
}

pub fn nixtract(
    flake_ref: impl Into<String>,
    system: Option<impl Into<String>>,
    attribute_path: Option<impl Into<String>>,
    offline: bool,
) -> Result<impl Iterator<Item = DerivationDescription>> {
    // Convert the arguments to the expected types
    let flake_ref = flake_ref.into();
    let system = system.map(Into::into);
    let attribute_path = attribute_path.map(Into::into);

    // Writes the `lib.nix` file to the tempdir and stores its path
    let lib = nix::lib::Lib::new()?;

    // Create a channel to communicate DerivationDescription to the main thread
    let (tx, rx) = mpsc::channel();

    log::info!(
        "Starting nixtract with flake_ref: {}, system: {}, attribute_path: {:?}",
        flake_ref,
        system
            .clone()
            .unwrap_or("builtins.currentSystem".to_owned()),
        attribute_path.clone().unwrap_or_default()
    );

    let collected_paths: Arc<Mutex<std::collections::HashSet<String>>> =
        Arc::new(Mutex::new(std::collections::HashSet::new()));

    // call find_attribute_paths to get the initial set of derivations
    let attribute_paths =
        nix::find_attribute_paths(&flake_ref, &system, &attribute_path, &offline, &lib)?;

    // Combine all AttributePaths into a single Vec
    let mut derivations: Vec<FoundDrv> = Vec::new();
    for attribute_path in attribute_paths {
        derivations.extend(attribute_path.found_drvs);
    }

    // Spawn a new rayon thread to call process on every foundDrv
    rayon::spawn(move || {
        derivations.into_par_iter().for_each(|found_drv| {
            match process(
                &collected_paths,
                &flake_ref,
                &system,
                found_drv.attribute_path,
                offline,
                &lib,
                tx.clone(),
            ) {
                Ok(_) => {}
                Err(e) => log::warn!("Error processing derivation: {}", e),
            }
        });
    });

    Ok(rx.into_iter())
}
