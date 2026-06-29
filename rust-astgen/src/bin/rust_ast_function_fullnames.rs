use anyhow::{Result, bail};
use clap::Parser;
use rust_ast_gen::config::RustAstGenConfig;
use rust_ast_gen::function_fullnames_gen;
use std::num::NonZero;
use std::path::PathBuf;
use std::thread::available_parallelism;

fn main() -> Result<()> {
    // We can use RUST_LOG={debug,info,trace,error,warn} in the environment
    // to control the log level.
    env_logger::init();

    let cli_args = CliArgs::parse();
    cli_args.validate()?;

    let config = config_from_args(cli_args)?;
    function_fullnames_gen::run(&config)
}

#[derive(Parser)]
#[clap(version)]
struct CliArgs {
    #[arg(help = "Input directory containing a Rust project")]
    #[arg(short = 'i', long = "input-dir")]
    input_dir: PathBuf,

    #[arg(help = "Skip sysroot loading. Faster, but will not resolve std symbols")]
    #[arg(long = "no-sysroot", default_value_t = false)]
    no_sysroot: bool,
}

impl CliArgs {
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

        Ok(())
    }
}

fn config_from_args(args: CliArgs) -> Result<RustAstGenConfig> {
    let input_dir_full_path = args.input_dir.canonicalize()?;
    let available_threads = available_parallelism().map(NonZero::get).unwrap_or(1);

    RustAstGenConfig::new(
        input_dir_full_path.clone(),
        input_dir_full_path,
        available_threads,
        !args.no_sysroot,
        false,
    )
}
