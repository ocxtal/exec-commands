mod config;
mod diff;
mod scan;

use crate::config::*;
use crate::diff::*;
use crate::scan::*;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use glob::glob;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(
    author = "Hajime Suzuki (suzuki.hajime.s@gmail.com)",
    version = "0.0.1",
    about = "scan markdown files and execute `console` blocks"
)]
struct Args {
    #[clap(help = "Input markdown files (overrides config and glob)")]
    inputs: Vec<PathBuf>,

    #[clap(short, long, help = "Remove existing output lines")]
    reverse: bool,

    #[clap(short, long, help = "Take diff between original and updated contents")]
    diff: bool,

    #[clap(
        short,
        long,
        value_name = "EXT",
        help = "Extension of files to scan (when no file specified by config or argument)",
        default_value = "md"
    )]
    extension: String,

    #[clap(
        long,
        value_name = "PWD",
        help = "Directory where commands are executed"
    )]
    pwd: Option<String>,

    #[clap(long, value_name = "PATH", help = "Additional paths to find commnands (colon-delimited)")]
    path: Option<String>,

    #[clap(
        short,
        long,
        value_name = "CONFIG",
        help = "Path to config file (it always loads .exec-commands.yaml if exists)"
    )]
    config: Option<String>,
}

fn glob_files(ext: &str) -> Result<Vec<PathBuf>> {
    let files = glob(&format!("./**/*.{}", ext))
        .with_context(|| format!("failed to glob files: \"*.{}\"", ext))?;

    Ok(files.map(|x| x.unwrap()).collect::<Vec<_>>())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // the default configuration file path is ./.exec-commands.yaml
    // unless specified by the command-line option
    let config = args
        .config
        .unwrap_or_else(|| ".exec-commands.yaml".to_string());
    let (inputs, config) = load_config(&config).unwrap_or((None, Config::default()));

    if args.reverse && args.diff {
        return Err(anyhow!("--reverse (-r) and --diff (-d) are exclusive."));
    }

    // --pwd and --path precedes over config; it overwrites the existing ones
    let mut config = config;
    if let Some(pwd) = args.pwd {
        config.pwd = compose_pwd(&pwd);
    }
    if let Some(path) = args.path {
        config.path = compose_path(&path);
    }

    // collect input files; argument > config > glob
    let inputs = if !args.inputs.is_empty() {
        args.inputs.clone()
    } else {
        inputs.unwrap_or(glob_files(&args.extension)?)
    };

    let mut has_diff = false;
    for file in &inputs {
        let original = std::fs::read_to_string(file)?;

        // first remove existing output lines
        let removed = remove_existing_command_outputs(&original)?;

        let added = if args.reverse {
            removed
        } else {
            insert_command_outputs(&removed, &config)?
        };

        if args.diff {
            has_diff |= print_diff(file.to_str().unwrap(), &original, &added)?;
        } else {
            std::fs::write(file, &added)?;
        }
    }

    if has_diff {
        std::process::exit(1);
    }

    Ok(())
}
