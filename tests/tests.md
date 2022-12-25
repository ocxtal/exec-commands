
# exec-commands test script

The following tests are executed with this configuration:

```console
$ cat ../.exec-commands.yaml
inputs:
  - "**/*.md"
pwd: "tests"
path: "target/debug"

```

## Scan only console blocks

```console
$ ls
tests.md
$ ls *.md
tests.md
```

``` console
$ ls
tests.md
```

```
$ ls
this output won't be updated as this block is not marked `console`.
```

## Working directory is preserved between commands

```console
$ ls
tests.md
$ cd ..
$ ls tests
tests.md
$ cd - >/dev/null
$ ls
tests.md
```

## "continued" attrubute preserves the environment

```console
$ export VAR0="var0"
$ export VAR1="var1"
$ echo "${VAR0} ${VAR1}"
var0 var1
```

```console continued
$ echo "${VAR0} ${VAR1}"
var0 var1
$ export VAR0="var2"
$ echo "${VAR0} ${VAR1}"
var2 var1
```

```console continued
$ echo "${VAR0} ${VAR1}"
var2 var1
```

```console continued and other attributes are ignored
$ echo "${VAR0} ${VAR1}"
var2 var1
```

```console the attribute continued can be mixed with other attributes
$ echo "${VAR0} ${VAR1}"
var2 var1
```

## Multiline commands

```console
$ ls \
tests.md
tests.md
$ ls \
  tests.md
tests.md
$ ls \
\
tests.md
tests.md
$ ls \
     \
  tests.md
tests.md
```

## Privileged commands (are executed as non-privileged)

```console
# ls
tests.md
```
