use anyhow::Result;
use glob::glob;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct AltCommand {
    raw: String,
    alt: String,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHooks {
    pre_block: Option<Vec<String>>,
    post_block: Option<Vec<String>>,
    pre_file: Option<Vec<String>>,
    post_file: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub pre_block: String,
    pub post_block: String,
    pub pre_file: String,
    pub post_file: String,
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
        let path = if let Some(path) = &raw.path {
            compose_path(path)
        } else {
            "".to_string()
        };

        // pre- and post-hooks
        let hooks = if let Some(hooks) = &raw.hooks {
            let join =
                |x: &Option<Vec<String>>| x.as_deref().map_or_else(String::new, |x| x.join("\n"));
            Hooks {
                pre_block: join(&hooks.pre_block),
                post_block: join(&hooks.post_block),
                pre_file: join(&hooks.pre_file),
                post_file: join(&hooks.post_file),
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

    let inputs = if let Some(raw_inputs) = &config.inputs {
        let mut inputs = Vec::new();
        for input in raw_inputs {
            let files = glob(input)?;
            inputs.extend(files.map(|x| x.unwrap()));
        }

        Some(inputs)
    } else {
        None
    };

    Ok((inputs, Config::from_raw(&config)))
}
