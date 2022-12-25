
# exec-command test script

## scan only console blocks

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

## directory preserved between commands

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

## continued

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

## multiline commands

```console
$ ls \
tests.md
tests.md
$ ls \
  tests.md
tests.md
```
