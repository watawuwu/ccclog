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
Generate changelog from git commit

USAGE:
    ccclog [FLAGS] [OPTIONS] [ARGS]

FLAGS:
    -e, --enable_email_link    Make a link to the author using git config.email
    -h, --help                 Prints help information
    -r, --reverse              Reverse commit display order
    -V, --version              Prints version information

OPTIONS:
    -i, --root_indent_level <root_indent_level>    Change markdown root subject indent [default: 2]

ARGS:
    <REPO_PATH>        Working directory of git [default: .]
    <REVISION_SPEC>    Revision spec. Ref to https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection
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
