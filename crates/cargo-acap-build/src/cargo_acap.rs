/// This module bridges the gap between `cargo` and `acap-build` using the application structure
/// conventions detailed in [`crate`].
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use acap_build::AppBuilder;
use anyhow::{bail, Context};
use log::{debug, error, warn};

use crate::{
    cargo::{get_cargo_metadata, json_message::JsonMessage},
    command_utils::RunWith,
    Architecture,
};

#[derive(Debug)]
pub enum Artifact {
    Eap { path: PathBuf, name: String },
    Exe { path: PathBuf },
}
pub fn build_and_pack(arch: Architecture, args: &[&str]) -> anyhow::Result<Vec<Artifact>> {
    // If user supplies a target we lose track of which target is currently being built
    assert!(!args.contains(&"--target"));

    let mut cargo = std::process::Command::new("cargo");
    cargo.arg("build");
    cargo.args(["--target", arch.triple()]);

    cargo.args(["--message-format", "json-render-diagnostics"]);

    // Allow the user to customize the behaviour in unanticipated or not-yet-supported ways.
    cargo.args(args);

    let mut messages = Vec::new();
    cargo.run_with_processed_stdout(|line| {
        match line {
            Ok(line) => match serde_json::from_str::<JsonMessage>(&line) {
                Ok(message) => messages.push(message),
                Err(e) => error!("Could not parse line because {e}"),
            },
            Err(e) => {
                error!("Could not take line because {e}");
                return Ok(());
            }
        }
        Ok(())
    })?;

    let cargo_target_directory = get_cargo_metadata()?.target_directory;
    let mut out_dirs = HashMap::new();
    let mut artifacts = Vec::new();
    for m in messages {
        match m {
            JsonMessage::CompilerArtifact {
                package_id,
                manifest_path,
                executable,
                target,
            } => {
                let Some(executable) = executable else {
                    debug!("Artifact is not an executable, skipping {package_id}");
                    continue;
                };
                let out_dir = out_dirs.get(&package_id).cloned();
                if is_app(&manifest_path, out_dir.as_deref()) {
                    // If the executable should be an ACAP app, create an `.eap` file.
                    artifacts.push(Artifact::Eap {
                        path: pack(
                            &cargo_target_directory,
                            arch,
                            target.name.clone(),
                            manifest_path,
                            executable,
                            out_dir,
                        )?,
                        name: target.name,
                    });
                } else {
                    // If the executable should not be an ACAP app, leave it as is.
                    artifacts.push(Artifact::Exe { path: executable });
                }
            }
            JsonMessage::CompilerMessage { message } => {
                // We expect these to be rendered to stderr when `--message-format` is
                // set to `json-render-diagnostics`, as opposed to `json`.
                error!("Received compiler-message: {message}")
            }
            JsonMessage::BuildFinished { success } => {
                debug!("Received build-finished message (success: {success})")
            }
            JsonMessage::BuildScriptExecuted {
                package_id,
                out_dir,
            } => {
                debug!("Received build-script-executed message for {package_id}");
                if let Some(out_dir) = out_dirs.insert(package_id, out_dir) {
                    warn!("Discarding out dir {out_dir:?}")
                }
            }
        }
    }
    Ok(artifacts)
}

fn pack(
    cargo_target_dir: &Path,
    arch: Architecture,
    package_name: String,
    manifest_path: PathBuf,
    executable: PathBuf,
    out_dir: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let mut staging_dir = cargo_target_dir.join(arch.nickname());
    if !staging_dir.is_dir() {
        std::fs::create_dir(&staging_dir)?;
    }
    staging_dir.push(
        executable
            .file_name()
            .context("built exe has no file name")?,
    );
    if staging_dir.is_dir() {
        std::fs::remove_dir_all(&staging_dir)?;
    }

    let manifest_dir = manifest_path
        .parent()
        .context("cargo manifest has no parent")?;

    let manifest = exactly_one(manifest_dir, out_dir.as_deref(), "manifest.json")?;
    debug!("Found manifest file: {manifest:?}");
    let license = exactly_one(manifest_dir, out_dir.as_deref(), "LICENSE")?;
    debug!("Found license file: {license:?}");

    debug!("Creating app builder");
    let mut app_builder = AppBuilder::new(
        staging_dir,
        arch,
        &package_name,
        &manifest,
        &executable,
        &license,
    )?;

    if let Some(d) = at_most_one(manifest_dir, out_dir.as_deref(), "additional-files")? {
        debug!("Found additional-files dir: {d:?}");
        app_builder.additional(&d)?;
    }
    if let Some(d) = at_most_one(manifest_dir, out_dir.as_deref(), "lib")? {
        debug!("Found lib dir: {d:?}");
        app_builder.lib(&d)?;
    }
    if let Some(d) = at_most_one(manifest_dir, out_dir.as_deref(), "html")? {
        debug!("Found html dir: {d:?}");
        app_builder.html(&d)?;
    }

    app_builder.build(env::var_os("ACAP_SDK_LOCATION").map(PathBuf::from))
}

fn exactly_one(
    manifest_dir: &Path,
    out_dir: Option<&Path>,
    file_name: &str,
) -> anyhow::Result<PathBuf> {
    let manifest_file = manifest_dir.join(file_name);
    let out_file = out_dir.map(|d| d.join(file_name));
    match (
        manifest_file.exists(),
        out_file.as_ref().map(|f| f.exists()).unwrap_or(false),
    ) {
        (false, false) => bail!("{file_name:?} exists neither in manifest dir {manifest_dir:?} nor in out dir {out_dir:?}"),
        (false, true) => Ok(out_file.expect("checked above")),
        (true, false) => Ok(manifest_file),
        (true, true) => bail!("{file_name:?} exist in both {manifest_dir:?} and {out_dir:?}"),
    }
}

fn at_most_one(
    manifest_dir: &Path,
    out_dir: Option<&Path>,
    file_name: &str,
) -> anyhow::Result<Option<PathBuf>> {
    let manifest_file = manifest_dir.join(file_name);
    let out_file = out_dir.map(|d| d.join(file_name));
    match (
        manifest_file.exists(),
        out_file.as_ref().map(|f| f.exists()).unwrap_or(false),
    ) {
        (false, false) => Ok(None),
        (false, true) => Ok(Some(out_file.expect("checked above"))),
        (true, false) => Ok(Some(manifest_file)),
        (true, true) => bail!("{file_name:?} exist in both {manifest_dir:?} and {out_dir:?}"),
    }
}

fn is_app(manifest_path: &Path, out_dir: Option<&Path>) -> bool {
    let manifest_dir = manifest_path.parent();
    if let Some(manifest_dir) = manifest_dir {
        if manifest_dir.join("manifest.json").is_file() {
            debug!("acap manifest found in {manifest_dir:?}");
            return true;
        }
    }

    if let Some(out_dir) = out_dir {
        if out_dir.join("manifest.json").is_file() {
            debug!("acap manifest found in {out_dir:?}");
            return true;
        }
    }

    debug!("acap manifest found  neither {manifest_dir:?} nor {out_dir:?}");
    false
}
