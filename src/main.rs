mod args;
mod fs;

use crate::args::Args;
use log::*;

use anyhow::Result;
use std::env;
use std::process::exit;

fn run(row_args: Vec<String>) -> Result<String> {
    let args = Args::new(&row_args)?;
    debug!("args: {:?}", args);

    Ok(args.some)
}

fn main() {
    pretty_env_logger::init();
    debug!("Started process");

    let args = env::args().collect::<Vec<String>>();
    let code = match run(args) {
        Ok(view) => {
            println!("{}", view);
            exitcode::OK
        }
        Err(err) => {
            eprintln!("{}", err);
            exitcode::USAGE
        }
    };
    exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    //use tempdir::TempDir;

    const BIN: &str = "ccclog";

    fn test_ok(row_args: Vec<&str>, expect: &str) -> Result<()> {
        let args = row_args.into_iter().map(String::from).collect();

        let actual = run(args)?;
        assert_eq!(actual, String::from(expect));

        Ok(())
    }

    #[test]
    fn ok() -> Result<()> {
        let expect = "ok";
        let arg = "ok";
        let args = vec![BIN, arg];
        test_ok(args, expect)
    }
}
