use crate::config::Config;
use Attr::*;

use anyhow::{anyhow, Result};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Attr {
    BaseText,
    ConsoleHeader,
    ConsoleHeaderContd,
    ConsoleCommand,
    ConsoleCommandEnd,
    ConsoleComment,
    ConsoleOutput,
    ConsoleFooter,
    OthersHeader,
    OthersContent,
    OthersFooter,
}

impl Attr {
    fn next(&self, line: &str) -> Attr {
        match *self {
            BaseText | ConsoleFooter | OthersFooter => self.parse_base_text(line),
            ConsoleHeader | ConsoleHeaderContd | ConsoleComment | ConsoleOutput
            | ConsoleCommandEnd => self.parse_command(line),
            ConsoleCommand => self.parse_command_continued(line),
            OthersHeader | OthersContent => self.parse_others(line),
        }
    }

    fn parse_base_text(&self, line: &str) -> Attr {
        if !line.starts_with("```") {
            return BaseText;
        }

        let rem = line.strip_prefix("```").unwrap().trim();
        if !rem.starts_with("console") {
            return OthersHeader;
        }

        if rem.split_whitespace().skip(1).any(|x| x == "continued") {
            ConsoleHeaderContd
        } else {
            ConsoleHeader
        }
    }

    fn parse_command_continued(&self, line: &str) -> Attr {
        if line.ends_with('\\') {
            return ConsoleCommand;
        }
        return ConsoleCommandEnd;
    }

    fn parse_command(&self, line: &str) -> Attr {
        if line.starts_with("$ ") || line.starts_with("# ") {
            return self.parse_command_continued(line);
        }
        if line.starts_with("  #") {
            return ConsoleComment;
        }
        if line.starts_with("```") {
            return ConsoleFooter;
        }
        ConsoleOutput
    }

    fn parse_others(&self, line: &str) -> Attr {
        if line.starts_with("```") {
            OthersFooter
        } else {
            OthersContent
        }
    }
}

fn annotate_lines(doc: &str) -> Vec<(&str, Attr)> {
    let mut attrs = Vec::new();
    let _ = doc.lines().fold(BaseText, |attr, line| {
        let next = attr.next(line);
        attrs.push((line, next));
        next
    });
    attrs
}

pub fn remove_existing_command_outputs(doc: &str) -> Result<String> {
    let annotation = annotate_lines(doc);

    let mut buf = String::new();
    for &(line, attr) in &annotation {
        if attr == ConsoleOutput {
            continue;
        }
        buf.push_str(line);
        buf.push('\n');
    }
    Ok(buf)
}

fn echo(line: &str, buf: &mut String) {
    let escaped = shell_escape::escape(line.into());
    buf.push_str("echo ");
    buf.push_str(&escaped);
    buf.push('\n');
}

fn build_commands(config: &Config, annotation: &[(&str, Attr)]) -> Vec<String> {
    let mut bin = vec![String::new()];
    let mut command = String::new();

    for &(line, attr) in annotation.iter() {
        if attr == ConsoleHeader {
            bin.push(String::new());
        }

        let buf = bin.last_mut().unwrap();
        match attr {
            ConsoleHeader | ConsoleHeaderContd | ConsoleComment | ConsoleFooter => {
                echo(line, buf);
            }
            ConsoleCommand | ConsoleCommandEnd => {
                echo(line, buf);

                let line = if line.starts_with("$ ") || line.starts_with("# ") {
                    &line[2..]
                } else {
                    line
                };
                command.push_str(line);
                command.push('\n');
            }
            _ => {}
        }
        if attr == ConsoleCommandEnd {
            let trimmed = command.trim();
            buf.push_str(
                config
                    .alt
                    .get(trimmed)
                    .map(|x| x.as_str())
                    .unwrap_or(trimmed),
            );
            buf.push('\n');
            command.clear();
        }
    }
    bin
}

trait Run {
    fn run(&self, raw_commands: &str, success: &mut bool) -> Result<Vec<u8>>;
}

impl Run for Config {
    fn run(&self, raw_commands: &str, success: &mut bool) -> Result<Vec<u8>> {
        let mut file = NamedTempFile::new()?;

        // let mut file = file.into_file();
        let header = format!(
            r#"
            #! /bin/bash
            set -eu -o pipefail
            export PATH={}
            cd {}
        "#,
            self.path,
            self.pwd.display()
        );
        file.write_all(header.as_bytes())?;
        file.write_all(raw_commands.as_bytes())?;

        let output = Command::new("bash").arg(file.path()).output()?;

        // FIXME: should we use logger?
        if !output.status.success() {
            eprintln!(
                "[exec-commands] \"bash {}\" exited with {}.\n{}\n{}\n{}",
                file.path().display(),
                output.status.code().unwrap(),
                raw_commands,
                std::str::from_utf8(&output.stderr).unwrap().trim(),
                std::str::from_utf8(&output.stdout).unwrap().trim(),
            );
            *success = false;
        }

        Ok(output.stdout)
    }
}

fn exec_commands(config: &Config, blocks: &[impl AsRef<str>]) -> Result<(bool, String)> {
    let mut success = true;
    let mut buf = String::new();

    config.run(&config.hooks.pre_file, &mut success)?;
    for block in blocks {
        config.run(&config.hooks.pre_block, &mut success)?;

        let output = config.run(block.as_ref(), &mut success)?;
        buf.push_str(std::str::from_utf8(&output).unwrap());

        config.run(&config.hooks.post_block, &mut success)?;
    }
    config.run(&config.hooks.post_file, &mut success)?;

    Ok((success, buf))
}

fn merge_outputs(annotation: &[(&str, Attr)], outputs: &str) -> Result<String> {
    let mut outputs = outputs.lines().peekable();
    let mut buf = String::new();

    for &(line, attr) in annotation {
        if matches!(attr, ConsoleHeader | ConsoleHeaderContd) {
            assert!(outputs.peek().unwrap().starts_with("```"));

            for line in &mut outputs {
                buf.push_str(line);
                buf.push('\n');

                if line == "```" {
                    break;
                }
            }
        }
        if matches!(attr, BaseText | OthersHeader | OthersContent | OthersFooter) {
            buf.push_str(line);
            buf.push('\n');
        }
    }

    Ok(buf)
}

pub fn insert_command_outputs(doc: &str, config: &Config) -> Result<(bool, String)> {
    let annotation = annotate_lines(doc);
    let commands = build_commands(config, &annotation);

    let (success, outputs) = exec_commands(config, &commands)?;
    if !success {
        return Err(anyhow!("the command block(s) above returned error."));
    }

    let output = merge_outputs(&annotation, &outputs)?;
    Ok((success, output))
}
