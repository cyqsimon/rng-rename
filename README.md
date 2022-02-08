# rng-rename
A CLI tool to rename files to randomly generated strings.

## Quick Start
```sh
# rename `path/to/foo` and `/path/to/bar.txt` to randomly generated names
rng-rename path/to/foo path/to/bar.txt
```

You can expect something like this:
```
Batch #1/1:
        "/abs/path/to/foo" -> "09c43d3d"
        "/abs/path/to/bar.txt" -> "67aec57d.txt"
Confirm batch? You can proceed(p), skip(s), or halt(h): proceed
Renamed 2 files. Done.
```
There are plenty of various options available. You can for example:
 - Preview using the `--dry-run` flag
 - Choose which character set to use for random names using the `--char-set` option
 - Choose upper/lower/mixed case where applicable using the `--case` option
 - Set a prefix and/or a suffix to the randomly generated name using `--prefix` and `--suffix` options
 - Choose what to do with the file extension using the `--ext-mode` option

And more. For full usage, run:
```sh
rng-rename --help
```
## Install

### from crates.io
[rng-rename on crates.io](https://crates.io/crates/rng-rename)

```sh
cargo install rng-rename
```

### from AUR
[rng-rename on AUR](https://aur.archlinux.org/packages/rng-rename)

```sh
# with paru
paru rng-rename

# or with yay
yay rng-rename
```
