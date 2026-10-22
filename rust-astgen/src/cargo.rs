use crate::config::RustAstGenConfig;
use anyhow::{Context, Result};
use log::{error, info, warn};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::{
    LoadCargoConfig, ProcMacroServerChoice, load_workspace as load_workspace_into_db,
};
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, ProjectManifest, ProjectWorkspace, RustLibSource,
};
use ra_ap_vfs::{AbsPathBuf, FileId, Vfs, VfsPath};

pub fn load_workspace(config: &RustAstGenConfig) -> Result<(RootDatabase, Vfs)> {
    let expand_proc_macros = config.expand_proc_macros && config.load_sysroot;

    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: expand_proc_macros,
        with_proc_macro_server: if expand_proc_macros {
            ProcMacroServerChoice::Sysroot
        } else {
            ProcMacroServerChoice::None
        },
        // doesn't seem necessary? slight runtime performance improvement by disabling it
        prefill_caches: false,
        proc_macro_processes: config.worker_threads,
        num_worker_threads: config.worker_threads,
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

    info!("manifest: {}", manifest);
    if let Some(target) = &config.target {
        info!("target: {}", target);
    }
    if !config.features.is_empty() || config.no_default_features {
        info!(
            "features: [{}], no-default-features={}",
            config.features.join(", "),
            config.no_default_features
        );
    }

    let mut workspace = ProjectWorkspace::load(manifest, cargo_config, &|progress_msg| {
        info!("progress: {}", progress_msg)
    })?;

    // proc-macro expansions requires running build scripts, which can fail.
    // We only log and proceed without expansion.
    if load_cargo_config.load_out_dirs_from_check {
        match workspace.run_build_scripts(cargo_config, &|progress_msg| {
            info!("progress: {}", progress_msg)
        }) {
            Ok(build_scripts) => {
                if let Some(reason) = build_scripts.error() {
                    warn!("build script error: {}", reason);
                }
                workspace.set_build_scripts(build_scripts);
            }
            Err(reason) => warn!("failed to run build scripts: {}", reason),
        }
    }

    if config.load_sysroot {
        match workspace.sysroot.error() {
            Some(reason) => error!("failed to load the Rust sysroot: {}", reason),
            None => {
                if let Some(sysroot_root) = workspace.sysroot.root() {
                    info!("sysroot: {}", sysroot_root);
                }
            }
        }
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
        .filter(|p| dunce::simplified(p.as_ref()).starts_with(&config.input_dir_full_path))
        .is_some();

    is_rust_file && is_inside_input_dir
}
