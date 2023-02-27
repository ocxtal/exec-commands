mod config;
mod diff;
mod scan;

use crate::config::*;
use crate::diff::*;
use crate::scan::*;

use anyhow::{anyhow, Context, Result};
use atty::Stream;
use clap::{ArgEnum, Parser};
use glob::glob;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(ArgEnum, Clone, Debug)]
enum Color {
    Auto,
    Never,
    Always,
}

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

    #[clap(
        long,
        value_name = "PATH",
        help = "Additional paths to find commands (colon-delimited)"
    )]
    path: Option<String>,

    #[clap(
        short,
        long,
        value_name = "CONFIG",
        help = "Path to config file (it always loads .exec-commands.yaml if exists)"
    )]
    config: Option<String>,

    #[clap(
        short = 'N',
        long = "ignore-default-config",
        help = "Prevent loading .exec-commands.yaml"
    )]
    ignore_default_config: bool,

    #[clap(
        arg_enum,
        long,
        value_name = "WHEN",
        help = "Colorize the output",
        default_value = "auto"
    )]
    color: Color,

    #[clap(long, value_name = "PAGER", help = "Feed the diff output to PAGER.")]
    pager: Option<String>,
}

fn set_output_color(args: &Args) {
    if matches!(args.color, Color::Always | Color::Never) {
        let enable = matches!(args.color, Color::Always);
        console::set_colors_enabled(enable);
        console::set_colors_enabled_stderr(enable);
    }
}

fn build_stdout(args: &Args) -> Result<(Option<Child>, Box<dyn Write>)> {
    let pager = args.pager.clone().or_else(|| std::env::var("PAGER").ok());
    if pager.is_none() && !atty::is(Stream::Stdout) {
        return Ok((None, Box::new(std::io::stdout())));
    }

    let pager = pager.unwrap_or_else(|| "less -S -F -R".to_string());
    let args: Vec<_> = pager.as_str().split_whitespace().collect();
    let mut child = Command::new(args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .spawn()?;

    let input = child
        .stdin
        .take()
        .context("failed to take stdin of the PAGER process")?;
    Ok((Some(child), Box::new(input)))
}

fn glob_files(ext: &str) -> Result<Vec<PathBuf>> {
    let files = glob(&format!("./**/*.{ext}"))
        .with_context(|| format!("failed to glob files: \"*.{ext}\""))?;

    Ok(files.map(|x| x.unwrap()).collect::<Vec<_>>())
}

fn build_config(args: &Args) -> Result<(Vec<PathBuf>, Config)> {
    // the default configuration file path is ./.exec-commands.yaml
    // unless specified by the command-line option
    let (inputs, config) = if let Some(config) = &args.config {
        load_config(config)?
    } else if !args.ignore_default_config && Path::new(".exec-commands.yaml").exists() {
        load_config(".exec-commands.yaml")?
    } else {
        (None, Config::default())
    };

    // collect input files; argument > config > glob
    let inputs = if !args.inputs.is_empty() {
        args.inputs.clone()
    } else {
        inputs.unwrap_or(glob_files(&args.extension)?)
    };

    // --pwd and --path precedes over config; it overwrites the existing ones
    let mut config = config;
    if let Some(pwd) = &args.pwd {
        config.pwd = compose_pwd(pwd);
    }
    if let Some(path) = &args.path {
        config.path = compose_path(path);
    }

    Ok((inputs, config))
}

fn scan_files(
    args: &Args,
    config: &Config,
    inputs: &[PathBuf],
    stdout: &mut impl Write,
) -> Result<bool> {
    let mut all_successful = true;

    for file in inputs {
        let original = std::fs::read_to_string(file)?;

        // first remove existing output lines
        let removed = remove_existing_command_outputs(&original)?;

        let added = if args.reverse {
            removed
        } else {
            let (success, added) = insert_command_outputs(&removed, config)?;
            all_successful &= success;

            added
        };

        if args.diff {
            all_successful &= !print_diff(file.to_str().unwrap(), &original, &added, stdout)?;
        } else {
            std::fs::write(file, &added)?;
        }
    }

    Ok(all_successful)
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.reverse && args.diff {
        return Err(anyhow!("--reverse (-r) and --diff (-d) are exclusive."));
    }

    set_output_color(&args);
    let (inputs, config) = build_config(&args)?;

    let (success, child) = {
        let (child, mut stdout) = build_stdout(&args)?;
        let success = scan_files(&args, &config, &inputs, &mut stdout)?;
        (success, child)
    };

    if let Some(mut child) = child {
        let _ = child.wait();
    }
    if !success {
        std::process::exit(1);
    }

    Ok(())
}
