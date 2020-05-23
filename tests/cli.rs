use anyhow::*;
use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::path::PathBuf;
use tar::Archive;
use tempfile::tempdir;

const BIN_NAME: &str = "ccclog";
const GIT_DATA1: &[u8] = include_bytes!("assets/git-data1.tar.gz");

fn cmd() -> Result<Command> {
    Ok(Command::cargo_bin(BIN_NAME)?)
}

pub fn git_dir() -> Result<PathBuf> {
    let tmp_dir = tempdir()?;
    let prefix = tmp_dir.into_path();

    let tar = GzDecoder::new(GIT_DATA1.as_ref());
    let mut archive = Archive::new(tar);
    archive.unpack(&prefix)?;
    Ok(prefix.join("git-data1"))
}

#[test]
fn help_err() -> Result<()> {
    let mut cmd = cmd()?;
    cmd.arg("-h");
    cmd.assert()
        .failure()
        .code(exitcode::USAGE)
        .stderr(predicate::str::contains("USAGE"));
    Ok(())
}

#[test]
fn not_found_git_repo_err() -> Result<()> {
    let mut cmd = cmd()?;
    let tmp_dir = tempdir()?;
    let path = tmp_dir.into_path();

    cmd.arg(path.to_str().unwrap());
    cmd.assert()
        .failure()
        .code(exitcode::USAGE)
        .stderr(predicate::str::contains("Not found git repository path"));
    Ok(())
}

#[test]
fn auto_detect_range_ok() -> Result<()> {
    let mut cmd = cmd()?;
    let dir = git_dir()?;
    cmd.args(&[dir.to_str().unwrap()]);
    cmd.assert().success().code(exitcode::OK).stdout(
        r#"## 0.2.0 - 2020-04-29
### Fix
- [6f90482] fix build script (Test User)

### Build
- [a673434] add build script (Test User)

### Feature
- [9cd3662] new fun (Test User)
"#,
    );

    Ok(())
}

#[test]
fn parse_range_ok() -> Result<()> {
    let mut cmd = cmd()?;
    let dir = git_dir()?;
    cmd.args(&[dir.to_str().unwrap(), "..0.1.0"]);
    cmd.assert().success().code(exitcode::OK).stdout(
        r#"## 0.1.0 - 2020-04-29
### Chore
- [9fa3647] add README (Test User)

### Feature
- [75a1b96] add first files (Test User)
"#,
    );

    Ok(())
}

#[test]
fn invalid_spec_ng() -> Result<()> {
    let mut cmd = cmd()?;
    let dir = git_dir()?;
    cmd.args(&[dir.to_str().unwrap(), "0.1.0"]);
    cmd.assert()
        .failure()
        .code(exitcode::USAGE)
        .stderr(predicate::str::contains("Don't support mode."));
    Ok(())
}

#[test]
fn invalid_option_ng() -> Result<()> {
    let mut cmd = cmd()?;
    cmd.args(&["--unknown"]);
    cmd.assert()
        .failure()
        .code(exitcode::USAGE)
        .stderr(predicate::str::contains("error: Found argument"));
    Ok(())
}
