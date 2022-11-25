
# exec-commands − scan markdown files and execute `console` blocks

**exec-commands** is a utility to update command-line-tool examples embedded in markdown files. It scans `.md`-suffixed files in a directory, extracts all code blocks with a `console` attribute, executes each `$`-prefixed line in a shell, and embeds the result right below the line.

**Note: Please use exec-commands inside a sandbox. exec-command itself doesn't have any mechanism to protect the host environment from malicious commands embedded in markdown files.**

## Options

```console
$ exec-commands --help
exec-commands 0.0.1
Hajime Suzuki (suzuki.hajime.s@gmail.com)
scan markdown files and execute `console` blocks

USAGE:
    exec-commands [OPTIONS] [INPUTS]...

ARGS:
    <INPUTS>...    Input markdown files (overrides config and glob)

OPTIONS:
    -c, --config <CONFIG>    Path to config file (it always loads .exec-commands.yaml if exists)
    -d, --diff               Take diff between original and updated contents
    -e, --extension <EXT>    Extension of files to scan (when no file specified by config or
                             argument) [default: md]
    -h, --help               Print help information
        --path <PATH>        Additional paths to find commands (colon-delimited)
        --pwd <PWD>          Directory where commands are executed
    -r, --reverse            Remove existing output lines
    -V, --version            Print version information
```

## Configuration file format

It takes configuration in the yaml format. Below is an example and description of the fields.

```yaml
# `inputs` is an array of input files.
inputs:
  - README.md
  - doc/**/*.md  	# wildcard allowed; `**` matches directories with zero or more depths.

# `pwd` specifies the directory to run commands
pwd: "test"

# `path` is additional directories to search commands; it expands environment variables.
path: "target/debug:$HOME/.cargo/bin"

# `alt` is a list of command substitutions. when it finds `raw`, it executes `alt` instead.
alt:
  - raw: "EDITOR=vim nd --inplace --patch-back=vipe quick.txt"
    alt: "nd --inplace --patch patch.txt quick.txt"

# it executes a sequence of commands before and after every block and file.
hooks:
  pre_block:
    - "git clean -f -d"
    - "git checkout HEAD ."
  post_block:
    - ":"
  pre_file:
    - ":"
  post_file:
    - "git clean -f -d"
    - "git checkout HEAD ."
```

## Copyright and License

2022, Hajime Suzuki. Licensed under MIT.
