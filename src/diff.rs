// this file is a fork of https://github.com/mitsuhiko/similar/blob/main/examples/terminal-inline.rs

use anyhow::Result;
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::fmt;
use std::io::Write;

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn print_diff(filename: &str, old: &str, new: &str, out: &mut impl Write) -> Result<bool> {
    let diff = TextDiff::from_lines(old, new);

    let mut has_diff = false;
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        has_diff = true;

        if idx == 0 {
            writeln!(out, "--- {filename}.original\n+++ {filename}.updated")?;
        } else {
            writeln!(out, "{:-^1$}", "-", 80)?;
        }

        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    out,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(out, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(out, "{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    writeln!(out)?;
                }
            }
        }
    }

    Ok(has_diff)
}
