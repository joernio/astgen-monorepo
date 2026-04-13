use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

pub struct RustAstGenConfig {
    pub(crate) input_dir_full_path: PathBuf,
    pub(crate) output_dir_full_path: PathBuf,
    pub(crate) cargo_worker_threads: usize,
}

impl RustAstGenConfig {
    pub fn new(
        input_dir_full_path: PathBuf,
        output_dir_full_path: PathBuf,
        num_threads: usize,
    ) -> Result<Self> {
        Self::ensure_paths_are_absolute(&input_dir_full_path, &output_dir_full_path)?;

        let config = Self {
            input_dir_full_path,
            output_dir_full_path,
            cargo_worker_threads: num_threads,
        };

        Ok(config)
    }

    fn ensure_paths_are_absolute(input_path: &Path, output_path: &Path) -> Result<()> {
        if !input_path.is_absolute() {
            bail!("input path must be absolute: {}", input_path.display());
        }

        if !output_path.is_absolute() {
            bail!("output path must be absolute: {}", output_path.display());
        }

        Ok(())
    }

    pub(crate) fn make_output_path_for_input_file(
        &self,
        input_file_path: &PathBuf,
    ) -> Result<PathBuf> {
        let relative_to_input_path = input_file_path
            .strip_prefix(&self.input_dir_full_path)
            .with_context(|| {
                format!(
                    "not able to relativize {} under {}",
                    input_file_path.display(),
                    self.input_dir_full_path.display()
                )
            })?;

        let output_file_full_path = self
            .output_dir_full_path
            .join(relative_to_input_path)
            .with_extension("json");

        Ok(output_file_full_path)
    }
}
