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
        "/abs/path/to/bar" -> "67aec57d.txt"
Confirm batch? You can proceed(p), skip(s), or halt(h): proceed
Renamed 2 files. Done.
```
There are plenty of various options available. You can for example:
 - Preview using the `--dry-run` flag
 - Choose which character set to use for random names using the `--char-set` option
 - Choose upper/lower/mixed case where applicable using the `--case` flag

And more. For full usage, run:
```sh
rng-rename -h
```
## Install

### with Cargo
```sh
cargo install rng-rename
```
