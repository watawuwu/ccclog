use anyhow::Result;
use structopt::StructOpt;

#[derive(StructOpt, PartialEq, Debug)]
struct RawArgs {
    #[structopt(name = "SOME")]
    some: Option<String>,
}

#[derive(PartialEq, Debug)]
pub struct Args {
    pub some: String,
}

impl Args {
    pub fn new(row_args: &[String]) -> Result<Args> {
        let mut app = RawArgs::clap();
        let mut buf: Vec<u8> = Vec::new();
        app.write_long_help(&mut buf)?;

        let clap = app.get_matches_from_safe(row_args)?;
        let row_args = RawArgs::from_clap(&clap);
        let some = match row_args.some {
            Some(s) => s,
            None => String::from("none"),
        };
        let args = Args { some };

        Ok(args)
    }
}
