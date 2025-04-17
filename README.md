# Update PR

This binary automatically updates a pull request in GitHub. It will check if the pull request is outdated and, if so, it will update it with the latest changes from the base branch.

## Installation

You can download the binary from the releases page of the GitHub repository. The binary is available for macOS.
Once downloaded included it in your `PATH`.

## Usage

```bash
update-pr
```

```bash
Usage: update-pr [OPTIONS] [WORKING_DIR]

Arguments:
  [WORKING_DIR]  Optional working directory

Options:
  -d <DELAY>      Delay between attempts. If this is not set, it will try only once. Examples: -d 10s -d 3m
  -h, --help      Print help
  -V, --version   Print version
```
