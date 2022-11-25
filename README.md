
# exec-commands − scan markdown files and execute `console` blocks

**exec-commands** is a utility to update command-line-tool examples embedded in markdown files. It scans `.md`-suffixed files in a directory, extracts all code blocks with a `console` attribute, executes each `$`-prefixed line in a shell, and embeds the result right below the line.

**注意: exec-commandsを使うときは、必ずsandboxedな環境の中で使用してください。exec-commandsはコマンドをwrapせずに実行するので、ホストの環境を悪意のあるコマンド列から保護することができません。**

## Options

```console
$ exec-commands --help
exec-commands 0.0.1
Hajime Suzuki (suzuki.hajime.s@gmail.com)
scan markdown files and execute `console` blocks

USAGE:
    exec-commands [OPTIONS] [INPUTS]...

ARGS:
    <INPUTS>...    Files to scan and execute `console` blocks

OPTIONS:
    -c, --config <CONFIG>    Path to config file (loads .exec-commands.yaml if exists)
    -d, --diff               Take diff between original and updated contents
    -e, --extension <EXT>    Extension of files to scan [default: md]
    -h, --help               Print help information
        --path <PATH>        Additional paths to find commnands (colon-delimited)
        --pwd <PWD>          Directory where commands are executed
    -r, --reverse            Remove existing output lines
    -V, --version            Print version information
```

## Configuration file format

設定ファイルはyaml形式です。以下のフィールドがあります。

* inputs: 入力markdownファイルのリストです。特定のファイルのみをスキャンするように明示する場合に使います。
* pwd: コマンドを実行する際のworking directoryです。
* path: コマンド (実行バイナリ) を探索するパスのリストです。Unixのパス形式 (コロンで連結される) です。
* alt: コマンド置換テーブルで、(raw, alt) のリストです。rawを発見したとき、代わりにaltを実行します。
  * raw: markdownファイルに現れる、もとのコマンドです
  * alt: rawの代わりに実行されるコマンドです。実行をスキップするときは `:` とします。
* hooks: ブロックやファイルの前後で実行されるhookを指定します。
  * pre_file: ファイルの先頭 (すべてのブロックをスキャンする前) で実行されるコマンドのリストです。
  * post_file: ファイルの末尾 (すべてのブロックをスキャンした後) で実行されるコマンドのリストです。
  * pre_block: ブロックの先頭 (ブロック内のコマンドを実行する前) で実行されるコマンドのリストです。
  * post_block: ブロックの末尾 (ブロック内のコマンドを実行した後) で実行されるコマンドのリストです。

以下に例を示します。

```yaml
inputs:
  - README.md

pwd: "test/"
path: "target/debug"

alt:
  - raw: "EDITOR=vim nd --inplace --patch-back=vipe quick.txt"
    alt: "nd --inplace --patch patch.txt quick.txt"

hooks:
  pre_block:
    - "echo pre-block"
    - "echo pre-block again"
```
