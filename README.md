# ccclog

Generate changelog from commit messages created by [Conventional Commits](https://www.conventionalcommits.org).

![Github Action](https://github.com/watawuwu/ccclog/workflows/Test/badge.svg?branch=master)
[![codecov](https://codecov.io/gh/watawuwu/ccclog/branch/master/graph/badge.svg)](https://codecov.io/gh/watawuwu/ccclog)
[![Latest version](https://img.shields.io/crates/v/ccclog.svg)](https://crates.io/crates/ccclog)
[![Documentation](https://docs.rs/ccclog/badge.svg)](https://docs.rs/crate/ccclog)
[![Docker](https://img.shields.io/docker/build/watawuwu/ccclog.svg)](https://cloud.docker.com/repository/docker/watawuwu/ccclog/)
![License](https://img.shields.io/crates/l/ccclog.svg)


## Sample
- https://github.com/watawuwu/ccclog/releases

## Getting Started

- Create a changelog from the latest tags in the working directory

```txt
❯❯ ccclog
## [0.0.0] - 2020-05-16
### Feat
- [[6274215]] add essential feature (Wataru Matsui)
- [[e4cff17]] add init (Wataru Matsui)

[0.0.0]: https://github.com/watawuwu/ccclog/compare/0eb9fad...0.0.0
[6274215]: https://github.com/watawuwu/ccclog/commit/62742151681860a5f9513510015035a8c0f6fdba
[e4cff17]: https://github.com/watawuwu/ccclog/commit/e4cff17b4c8b7103cea4e36eb34dd539937af4ba
```

- Other usage

```txt
USAGE:
    ccclog [FLAGS] [OPTIONS] [--] [ARGS]

FLAGS:
    -e, --enable-email-link    Make a link to the author using git config.email
    -h, --help                 Prints help information
    -r, --reverse              Reverse commit display order
    -V, --version              Prints version information

OPTIONS:
    -s, --ignore-summary <ignore-summary>
            Ignore summary use regex. Syntax: https://docs.rs/regex/1.3.7/regex/#syntax

    -t, --ignore-types <ignore-types>...
            Ignore commit type. ex) feat|fix|build|doc|chore|ci|style|refactor|perf|test

    -i, --root-indent-level <root-indent-level>    Change markdown root subject indent [default: 2]
    -m, --tag-pattern <tag-pattern>
            Regular expression that matches the tag [default: ^v?\d+?.\d+?.\d+?(\-[\w.-]+?)?(\+[\w.-]+?)?$]


ARGS:
    <REPO_PATH>        Working directory of git [default: ]
    <REVISION_SPEC>    Revision spec. Ref to https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection
```

## Usage from Github Action

### Inputs

#### `repo_path`

Working directory of git.

#### `revision_spec`

Revision spec. Ref to https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection

#### `options`

ccclog command option. Options that can be specified on the command line can be specified as they are (multiple options allowed).
ex) `--reverse  --tag-pattern='prefix-.+'`

### Outputs

#### `changelog`

Generated changelog

### Example usage

``` yaml
name: Github Action Sample

on:
  push:
    tags:
     - '*.*.*'

jobs:
  create_github_release:
    name: Create Github release
    # If you use actions, use the platform that docker works
    runs-on: ubuntu-latest

    steps:
    - name: Setup code
      uses: actions/checkout@v2
      with:
        # Set the required number to create a changelog.
        # If the history is small, it is recommended to specify 0 and acquire all history.
        fetch-depth: 0

    - name: Fetch all tags
      # Get the tags to create history
      # `actions/checkout@v2` has the --not-tags option
      run: git fetch origin +refs/tags/*:refs/tags/*

    - name: Create Changelog
      id: create_changelog
      uses: watawuwu/ccclog@gha-v0
      # with:
      #   repo_path: "."
      #   revision_spec: "..HEAD"
      #   options: '--root-indent-level=3 --tag-pattern=component-v --ignore-summary=cargo\srelease.+'

    # You can also exec the binary
    # - name: Create Changelog
    #   id: create_changelog
    #   shell: bash -x {0}
    #   run: |
    #     mkdir bin && curl --tlsv1.2 -sSf https://raw.githubusercontent.com/watawuwu/ccclog/master/install.sh | sh -s -- --prefix ./
    #     changelog="$(bin/ccclog)"
    #     changelog="${changelog//$'\n'/'%0A'}"
    #     echo "::set-output name=changelog::$changelog"

    - name: Create release
      id: create_release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ github.ref }}
        release_name: ${{ github.ref }}
        draft: false
        prerelease: false
        # outputs.changelog parameter is available
        body: ${{ steps.create_changelog.outputs.changelog }}
```

## Installing

- Install binary directly

```
❯❯ curl --tlsv1.2 -sSf https://raw.githubusercontent.com/watawuwu/ccclog/master/install.sh | sh
```

- Compile and install

```
❯❯ git clone https://github.com/watawuwu/ccclog.git && cd ccclog

❯❯ make install
```

- Install with cargo

```
❯❯ cargo install ccclog
```

## Contributing

Please read [CONTRIBUTING.md](https://gist.github.com/PurpleBooth/b24679402957c63ec426) for details on our code of conduct, and the process for submitting pull requests to us.

## Versioningx

We use [SemVer](http://semver.org/) for versioning.

## License
This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Authors

- Wataru Matsui
