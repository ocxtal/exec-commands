mod config;
mod scan;

use crate::config::*;
use crate::scan::*;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use glob::glob;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[clap(author = "Hajime Suzuki (suzuki.hajime.s@gmail.com)", version = "0.0.1", about = "scan markdown files and execute `console` blocks")]
struct Args {
    #[clap(help = "Files to scan and execute `console` blocks")]
    inputs: Vec<PathBuf>,

    #[clap(short, long, help = "Remove existing output lines")]
    reverse: bool,

    #[clap(long, help = "Only check if the files will be updated")]
    check: bool,

    #[clap(short, long, value_name = "EXT", help = "Extension of files to scan", default_value = "md")]
    extension: String,

    #[clap(
        long,
        value_name = "PWD",
        help = "Directory where commands are executed"
    )]
    pwd: Option<String>,

    #[clap(
        long,
        value_name = "PATH",
        help = "Additional paths to find commnands"
    )]
    path: Option<String>,

    #[clap(
        short,
        long,
        value_name = "CONFIG",
        help = "Path to config file (loads .exec-commands.yaml if exists)"
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
    let config = args.config.unwrap_or(".exec-commands.yaml".to_string());
    let (inputs, config) = load_config(&config).unwrap_or((None, Config::default()));

    if args.reverse && args.check {
        return Err(anyhow!("--reverse (-r) and --check are exclusive."));
    }

    let mut config = config;
    if let Some(pwd) = args.pwd {
        config.pwd = PathBuf::from_str(&pwd).unwrap();
    }
    if let Some(path) = args.path {
        config.path = path.to_string();
    }

    // collect input files; argument > config > glob
    let inputs = if !args.inputs.is_empty() {
        args.inputs.clone()
    } else {
        inputs.unwrap_or(glob_files(&args.extension)?)
    };

    for file in &inputs {
        let original = std::fs::read_to_string(file)?;

        // first remove existing output lines
        let removed = remove_existing_command_outputs(&original)?;

        let added = if args.reverse {
            removed
        } else {
            insert_command_outputs(&removed, &config)?
        };

        if args.check {
            assert_eq!(&original, &added);
        } else {
            std::fs::write(file, &added)?;
        }
    }

    Ok(())
}
