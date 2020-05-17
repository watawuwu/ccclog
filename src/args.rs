use crate::git::CommitType;
use anyhow::Result;
use regex::Regex;
use structopt::{clap, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Args {
    #[structopt(short, long, help = "Make a link to the author using git config.email")]
    pub enable_email_link: bool,
    #[structopt(short, long, help = "Reverse commit display order")]
    pub reverse: bool,
    #[structopt(
        short = "i",
        long,
        default_value = "2",
        help = "Change markdown root subject indent"
    )]
    pub root_indent_level: u8,
    #[structopt(
        short = "s",
        long,
        help = "Ignore summary use regex. Syntax: https://docs.rs/regex/1.3.7/regex/#syntax"
    )]
    pub ignore_summary: Option<Regex>,
    #[structopt(
        short = "t",
        long,
        help = "Ignore commit type. ex) feat|fix|build|doc|chore|ci|style|refactor|perf|test"
    )]
    pub ignore_types: Option<Vec<CommitType>>,
    #[structopt(
        name = "REPO_PATH",
        default_value = ".",
        help = "Working directory of git"
    )]
    pub path: String,
    #[structopt(
        name = "REVISION_SPEC",
        help = "Revision spec. Ref to https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection"
    )]
    revspec: Option<String>,
}

impl Args {
    pub fn new(args: &[String]) -> Result<Args> {
        let app = Args::clap();
        let clap = app.get_matches_from_safe(args)?;
        Ok(Args::from_clap(&clap))
    }

    pub fn revspec(&self) -> Option<&str> {
        self.revspec.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIN: &str = "ccclog";

    fn to_string(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(String::from).collect()
    }

    #[test]
    fn args_ok() -> Result<()> {
        let args = to_string(vec![BIN]);
        assert!(Args::new(&args).is_ok());

        let args = to_string(vec![BIN, "."]);
        assert!(Args::new(&args).is_ok());

        let args = to_string(vec![BIN, ".", "0.1.0..0.2.0"]);
        assert!(Args::new(&args).is_ok());

        Ok(())
    }

    #[test]
    fn args_err() -> Result<()> {
        let args = to_string(vec![BIN, "-h"]);
        let err = Args::new(&args).unwrap_err();
        if let Some(err) = err.downcast_ref::<structopt::clap::Error>() {
            assert_eq!(err.kind, structopt::clap::ErrorKind::HelpDisplayed);
            assert!(err.to_string().contains("USAGE"));
        }

        let args = to_string(vec![BIN, "-V"]);
        let err = Args::new(&args).unwrap_err();
        if let Some(err) = err.downcast_ref::<structopt::clap::Error>() {
            // Error doesn't include version info
            // Clap printed?
            assert_eq!(err.kind, structopt::clap::ErrorKind::VersionDisplayed);
        }
        Ok(())
    }
}
