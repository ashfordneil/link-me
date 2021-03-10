# link-me

A small CLI utility to generate links to a piece of code on your computer. You
give it a file name, and it'll give you a github URL to that file on your
current branch.

## Usage
Here's the `--help` output for the program:

```
link-me 0.1.0
Get a shareable link to a section of source code

USAGE:
    link-me [OPTIONS] <file-path> --ref-type <ref-type>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --line-number <line-number>    Which line in that file do you want to link to?
    -r, --ref-type <ref-type>          Which way do you want to link to this? Options are "branch" and "commit"

ARGS:
    <file-path>    Which file do you want to link to?
```

And here's an example:

```shell
$ link-me README.md --ref-type branch --line-number 33
https://github.com/ashfordneil/link-me/blob/main/README.md#L33
```