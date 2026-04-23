use anyhow::{Context, Result, bail};
use clap::Parser;
use rust_ast_gen::config::RustAstGenConfig;
use rust_ast_gen::json_gen;
use std::num::NonZero;
use std::path::PathBuf;
use std::thread::available_parallelism;

fn main() -> Result<()> {
    // We can use RUST_LOG={debug,info,trace,error,warn} in the environment
    // to control the log level.
    env_logger::init();

    // `parse` will exit the program if there are any errors.
    let cli_args = RustAstGenCliArgs::parse();

    let config = RustAstGenConfig::try_from(cli_args)?;

    json_gen::run(&config)
}

#[derive(Parser)]
#[clap(version)]
struct RustAstGenCliArgs {
    #[arg(help = "Input directory containing a Rust project")]
    #[arg(short = 'i', long = "input-dir")]
    input_dir: PathBuf,

    #[arg(help = "Output directory where generated files will be written to")]
    #[arg(short = 'o', long = "output-dir")]
    output_dir: PathBuf,
}

impl RustAstGenCliArgs {
    fn validate(&self) -> Result<()> {
        if !self.input_dir.exists() {
            bail!("input path does not exist: {}", self.input_dir.display());
        }

        if !self.input_dir.is_dir() {
            bail!(
                "input path is not a directory: {}",
                self.input_dir.display()
            );
        }

        if self.output_dir.exists() && !self.output_dir.is_dir() {
            bail!(
                "output path is not a directory: {}",
                self.output_dir.display()
            );
        }

        Ok(())
    }

    /// Confirm that both input and output directories exist.
    /// Will attempt to create the output directory if it does not exist.
    pub fn ensure_provided_dirs_exist(&self) -> Result<()> {
        self.validate()?;

        std::fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "failed to create output directory: {}",
                self.output_dir.display()
            )
        })
    }
}

impl TryFrom<RustAstGenCliArgs> for RustAstGenConfig {
    type Error = anyhow::Error;

    fn try_from(args: RustAstGenCliArgs) -> Result<Self> {
        args.ensure_provided_dirs_exist()?;

        let input_dir_full_path = args.input_dir.canonicalize()?;
        let output_dir_full_path = args.output_dir.canonicalize()?;
        let available_threads = available_parallelism().map(NonZero::get).unwrap_or(1);

        RustAstGenConfig::new(input_dir_full_path, output_dir_full_path, available_threads)
    }
}
