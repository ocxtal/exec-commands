use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq, Deserialize)]
struct AltCommand {
    raw: String,
    alt: String,
}

#[derive(Debug, PartialEq, Deserialize)]
struct RawHooks {
    pre_block: Option<Vec<String>>,
    post_block: Option<Vec<String>>,
    pre_file: Option<Vec<String>>,
    post_file: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct RawConfig {
    // input markdown files
    inputs: Option<Vec<String>>,

    // a directory where commands are executed
    pwd: Option<String>,

    // colon-separated paths to search commands
    path: Option<String>,

    // alternative commands
    alt: Option<Vec<AltCommand>>,

    // pre- and post-hooks
    hooks: Option<RawHooks>,
}

#[derive(Debug, Default)]
pub struct Hooks {
    pub pre_block: Vec<String>,
    pub post_block: Vec<String>,
    pub pre_file: Vec<String>,
    pub post_file: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    pub pwd: PathBuf,
    pub path: String,
    pub alt: HashMap<String, String>,
    pub hooks: Hooks,
}

impl Default for Config {
    fn default() -> Self {
        let pwd = std::env::current_dir().unwrap();
        let path = std::env::var("PATH").unwrap_or_default();

        Config {
            pwd,
            path,
            alt: HashMap::new(),
            hooks: Hooks::default(),
        }
    }
}

pub fn compose_pwd(pwd: &str) -> PathBuf {
    std::fs::canonicalize(PathBuf::from(pwd)).unwrap()
}

pub fn compose_path(path: &str) -> String {
    // convert colon-delimited paths to absolute ones
    let path = path
        .split(':')
        .map(|x| std::fs::canonicalize(x).unwrap())
        .collect::<Vec<_>>();

    let mut buf = String::new();
    for x in &path {
        buf.push_str(x.to_str().unwrap());
        buf.push(':');
    }

    // and append environment paths at the tail
    let env_path = std::env::var("PATH").unwrap_or_default();
    buf.push_str(&env_path);

    buf
}

impl Config {
    fn from_raw(raw: &RawConfig) -> Self {
        // alternative command map
        let alt = raw.alt.as_ref().map_or(HashMap::new(), |x| {
            x.iter()
                .map(|x| (x.raw.clone(), x.alt.clone()))
                .collect::<HashMap<String, String>>()
        });

        // current working directory
        let pwd = if let Some(pwd) = &raw.pwd {
            compose_pwd(pwd)
        } else {
            std::env::current_dir().unwrap()
        };

        // unix-style search paths
        let path = compose_path(raw.path.as_deref().unwrap_or(""));

        // pre- and post-hooks
        let hooks = if let Some(hooks) = &raw.hooks {
            let clone_or_default = |x: &Option<Vec<String>>| -> Vec<String> {
                if let Some(x) = x {
                    x.clone()
                } else {
                    Vec::new()
                }
            };

            Hooks {
                pre_block: clone_or_default(&hooks.pre_block),
                post_block: clone_or_default(&hooks.post_block),
                pre_file: clone_or_default(&hooks.pre_file),
                post_file: clone_or_default(&hooks.post_file),
            }
        } else {
            Hooks::default()
        };

        Config {
            pwd,
            path,
            alt,
            hooks,
        }
    }
}

pub fn load_config(config: &str) -> Result<(Option<Vec<PathBuf>>, Config)> {
    let config = std::fs::read_to_string(config)?;
    let config: RawConfig = serde_yaml::from_str(&config)?;

    let inputs = config.inputs.as_ref().map(|inputs| {
        inputs
            .iter()
            .map(|x| PathBuf::from_str(x).unwrap())
            .collect::<Vec<_>>()
    });

    Ok((inputs, Config::from_raw(&config)))
}
