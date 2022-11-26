use crate::config::Config;
use anyhow::{anyhow, Result};
use std::process::Command;

enum State {
    OutOfBlock,
    InsideBlock,
}

#[derive(Eq, PartialEq)]
enum BlockHook {
    None,
    Pre,
    Post,
}

impl State {
    fn new() -> Self {
        State::OutOfBlock
    }

    /// returns (keep, clear, command)
    fn update<'a>(&mut self, line: &'a str) -> (bool, BlockHook, Option<&'a str>) {
        match self {
            State::OutOfBlock => {
                if line.starts_with("```console") {
                    *self = State::InsideBlock;
                    return (true, BlockHook::Pre, None);
                }
                (true, BlockHook::None, None)
            }
            State::InsideBlock => {
                if line == "```" {
                    *self = State::OutOfBlock;
                    return (true, BlockHook::Post, None);
                }
                let is_command = line.starts_with('$');
                let keep = is_command | line.starts_with("  #");
                let command = if is_command {
                    Some(line[1..].trim())
                } else {
                    None
                };

                (keep, BlockHook::None, command)
            }
        }
    }
}

trait RunCommand {
    fn run(&self, command: &str) -> Result<(bool, Vec<u8>)>;
    fn pre_block_hook(&self) -> Result<()>;
    fn post_block_hook(&self) -> Result<()>;
    fn pre_file_hook(&self) -> Result<()>;
    fn post_file_hook(&self) -> Result<()>;
}

impl RunCommand for Config {
    fn run(&self, raw_command: &str) -> Result<(bool, Vec<u8>)> {
        let command = &format!(
            "PATH={}; cd {}; {}",
            self.path,
            self.pwd.to_str().unwrap(),
            raw_command
        );
        let output = Command::new("bash").args(["-c", command]).output()?;

        // FIXME: should we use logger?
        if !output.status.success() {
            eprintln!(
                "[exec-commands] {:?} returned {}.\n{}",
                raw_command,
                output.status.code().unwrap(),
                std::str::from_utf8(&output.stderr).unwrap().trim()
            );
        }

        Ok((output.status.success(), output.stdout))
    }

    fn pre_block_hook(&self) -> Result<()> {
        for command in &self.hooks.pre_block {
            // note: any error in hooks is regarded as a fatal error
            if !self.run(command)?.0 {
                return Err(anyhow!("aborting pre_block_hook..."));
            }
        }
        Ok(())
    }

    fn post_block_hook(&self) -> Result<()> {
        for command in &self.hooks.post_block {
            if !self.run(command)?.0 {
                return Err(anyhow!("aborting post_block_hook..."));
            }
        }
        Ok(())
    }

    fn pre_file_hook(&self) -> Result<()> {
        for command in &self.hooks.pre_file {
            if !self.run(command)?.0 {
                return Err(anyhow!("aborting pre_file_hook..."));
            }
        }
        Ok(())
    }

    fn post_file_hook(&self) -> Result<()> {
        for command in &self.hooks.post_file {
            if !self.run(command)?.0 {
                return Err(anyhow!("aborting post_file_hook..."));
            }
        }
        Ok(())
    }
}

pub fn remove_existing_command_outputs(contents: &str) -> Result<String> {
    let mut state = State::new();
    let mut filtered = String::new();

    for line in contents.lines() {
        let (keep, _, _) = state.update(line);
        if keep {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }

    Ok(filtered)
}

pub fn insert_command_outputs(contents: &str, config: &Config) -> Result<(bool, String)> {
    let mut state = State::new();
    let mut inserted = String::new();

    config.pre_file_hook()?;

    let mut all_successful = true;
    for line in contents.lines() {
        let (keep, hook, command) = state.update(line);

        if keep {
            inserted.push_str(line);
            inserted.push('\n');
        }

        if hook == BlockHook::Pre {
            config.pre_block_hook()?;
        }

        if let Some(command) = command {
            let command = if let Some(alt_command) = config.alt.get(command) {
                alt_command
            } else {
                command
            };

            let (success, output) = config.run(command)?;
            all_successful &= success;

            // collect output to buf; supplement trailing \n if missing
            inserted.push_str(std::str::from_utf8(&output).unwrap());
            if !output.is_empty() && output.last() != Some(&b'\n') {
                inserted.push('\n');
            }
        }

        if hook == BlockHook::Post {
            config.post_block_hook()?;
        }
    }
    config.post_file_hook()?;

    Ok((all_successful, inserted))
}
