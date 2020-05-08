#[macro_use]
extern crate strum_macros;

mod args;
mod changelog;
mod git;

use crate::args::Args;
use log::*;

use crate::changelog::{Changelog, Config};
use anyhow::*;
use std::env;
use std::process::exit;

fn run(args: Vec<String>) -> Result<String> {
    let args = Args::new(&args)?;
    debug!("args: {:?}", args);

    let repo = git::repo(&args.path)?;
    let commits = git::commits(&repo, args.revspec())?;

    let config = Config {
        enable_email_link: args.enable_email_link,
        reverse: args.reverse,
        root_indent_level: args.root_indent_level,
    };
    let changelog = Changelog::from(config);
    let url = git::gurl(&repo);
    let markdown = changelog.markdown(url.as_ref(), &commits)?;
    Ok(markdown)
}

fn main() {
    pretty_env_logger::init();
    let args = env::args().collect::<Vec<String>>();
    let code = match run(args) {
        Ok(markdown) => {
            println!("{}", markdown);
            exitcode::OK
        }
        Err(err) => {
            eprintln!("{:?}", err);
            exitcode::USAGE
        }
    };
    exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::git_dir;
    use anyhow::{Context, Result};

    const BIN: &str = "ccclog";

    fn test_ok(args: Vec<&str>, expect: &str) -> Result<()> {
        let args = args.into_iter().map(String::from).collect::<Vec<String>>();

        let actual = run(args)?;
        assert_eq!(actual, expect);

        Ok(())
    }

    #[test]
    fn ok() -> Result<()> {
        let dir = git_dir()?;
        let dir = dir.to_str().context("Failed to change PathBuf to &str")?;
        let args = vec![BIN, dir];

        let expect = r#"## 0.2.0 - 2020-04-29
### Fix
- [6f90482] fix build script (Test User)

### Build
- [a673434] add build script (Test User)

### Feature
- [9cd3662] new fun (Test User)
"#;
        test_ok(args, expect)
    }
}
