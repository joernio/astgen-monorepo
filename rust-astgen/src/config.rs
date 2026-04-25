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
        input_file_path: &Path,
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
            .with_added_extension("json");

        Ok(output_file_full_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_make_output_path_for_input_file_success_unix() -> Result<()> {
        let input_dir = PathBuf::from("/input");
        let output_dir = PathBuf::from("/output");
        let config = RustAstGenConfig::new(input_dir.clone(), output_dir.clone(), 1)?;

        let input_file = input_dir.join("subdir").join("file.rs");
        let output_file = config.make_output_path_for_input_file(&input_file)?;

        let expected_output_file = output_dir.join("subdir").join("file.rs.json");
        assert_eq!(output_file, expected_output_file);

        Ok(())
    }

    #[test]
    #[cfg(windows)]
    fn test_make_output_path_for_input_file_success_windows() -> Result<()> {
        let input_dir = PathBuf::from(r"C:\input");
        let output_dir = PathBuf::from(r"C:\output");
        let config = RustAstGenConfig::new(input_dir.clone(), output_dir.clone(), 1)?;

        let input_file = input_dir.join("subdir").join("file.rs");
        let output_file = config.make_output_path_for_input_file(&input_file)?;

        let expected_output_file = output_dir.join("subdir").join("file.rs.json");
        assert_eq!(output_file, expected_output_file);

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_make_output_path_for_input_file_not_under_input_dir_unix() -> Result<()> {
        let input_dir = PathBuf::from("/input");
        let output_dir = PathBuf::from("/output");
        let config = RustAstGenConfig::new(input_dir, output_dir, 1)?;

        let other_file = PathBuf::from("/other/file.rs");
        let result = config.make_output_path_for_input_file(&other_file);

        assert!(result.is_err());
        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("not able to relativize"));

        Ok(())
    }

    #[test]
    #[cfg(windows)]
    fn test_make_output_path_for_input_file_not_under_input_dir_windows() -> Result<()> {
        let input_dir = PathBuf::from(r"C:\input");
        let output_dir = PathBuf::from(r"C:\output");
        let config = RustAstGenConfig::new(input_dir, output_dir, 1)?;

        let other_file = PathBuf::from(r"C:\other\file.rs");
        let result = config.make_output_path_for_input_file(&other_file);

        assert!(result.is_err());
        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("not able to relativize"));

        Ok(())
    }
}
