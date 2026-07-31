use crate::config::RustAstGenConfig;
use anyhow::{Context, Result};
use log::{error, info};
use ra_ap_ide::RootDatabase;
use ra_ap_load_cargo::{
    LoadCargoConfig, ProcMacroServerChoice, load_workspace as load_workspace_into_db,
};
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, ProjectManifest, ProjectWorkspace, RustLibSource,
};
use ra_ap_vfs::{AbsPathBuf, FileId, Vfs, VfsPath};
use std::path::Path;

pub(crate) fn load_workspace(config: &RustAstGenConfig) -> Result<(RootDatabase, Vfs)> {
    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        // doesn't seem necessary? slight runtime performance improvement by disabling it
        prefill_caches: false,
        proc_macro_processes: 0,
        num_worker_threads: config.cargo_worker_threads,
    };

    let cargo_config = CargoConfig {
        sysroot: if config.load_sysroot {
            RustLibSource::Discover.into()
        } else {
            None
        },
        target: config.target.clone(),
        features: CargoFeatures::Selected {
            features: config.features.clone(),
            no_default_features: config.no_default_features,
        },
        ..CargoConfig::default()
    };

    info!(
        "loading workspace using {} threads: {}",
        load_cargo_config.num_worker_threads,
        config.input_dir_full_path.display()
    );

    let (root_db, vfs) = load_workspace_at(config, &cargo_config, &load_cargo_config)
        .with_context(|| {
            format!(
                "failed to load the Rust project at `{}`. Are `cargo` and `rustc` on your PATH?",
                config.input_dir_full_path.display()
            )
        })?;

    Ok((root_db, vfs))
}

fn load_workspace_at(
    config: &RustAstGenConfig,
    cargo_config: &CargoConfig,
    load_cargo_config: &LoadCargoConfig,
) -> Result<(RootDatabase, Vfs)> {
    let root = AbsPathBuf::assert_utf8(config.input_dir_full_path.clone());
    let manifest = ProjectManifest::discover_single(&root)?;
    let workspace = ProjectWorkspace::load(manifest, cargo_config, &|progress_msg| {
        info!("progress: {}", progress_msg)
    })?;

    if config.load_sysroot
        && let Some(reason) = workspace.sysroot.error()
    {
        error!("failed to load the Rust sysroot: {}", reason);
    }

    let (root_db, vfs, _) =
        load_workspace_into_db(workspace, &cargo_config.extra_env, load_cargo_config)?;

    Ok((root_db, vfs))
}

pub(crate) fn collect_input_files(
    config: &RustAstGenConfig,
    vfs: &Vfs,
) -> Result<Vec<(FileId, VfsPath)>> {
    let mut result = Vec::new();
    let mut entries = 0usize;

    for (file_id, vfs_path) in vfs.iter() {
        entries += 1;

        if should_collect_file(config, vfs_path) {
            result.push((file_id, vfs_path.clone()));
        }
    }

    info!("collected {} files out of {} found", result.len(), entries);
    Ok(result)
}

fn should_collect_file(config: &RustAstGenConfig, vfs_path: &VfsPath) -> bool {
    let vfs_path = vfs_path.as_path();

    let is_rust_file = vfs_path.filter(|p| p.extension() == Some("rs")).is_some();

    let is_inside_input_dir = vfs_path
        .filter(|p| AsRef::<Path>::as_ref(p).starts_with(&config.input_dir_full_path))
        .is_some();

    is_rust_file && is_inside_input_dir
}
