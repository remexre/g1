use anyhow::Result;
use g1_common::{nameless::NamelessQuery, query::Query};
use std::{io::Read, path::PathBuf};

#[paw::main]
fn main(args: Args) -> Result<()> {
    femme::start(match args.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    })?;

    match args.subcommand {
        Subcommand::ValidateQuery { path } => {
            let src = match path {
                Some(path) => std::fs::read_to_string(path)?,
                None => {
                    let mut src = String::new();
                    std::io::stdin().read_to_string(&mut src)?;
                    src
                }
            };
            let query = src.parse::<Query>()?;
            let query = NamelessQuery::from_query::<g1_common::SimpleError>(query)?;
            println!("{:#?}", query);
            Ok(())
        }
    }
}

/// A command-line tool for experimenting with G1 and manually interacting with it.
#[derive(Debug, structopt::StructOpt)]
struct Args {
    /// Increases the verbosity of logging.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: usize,

    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, structopt::StructOpt)]
enum Subcommand {
    /// Validates that a query is valid.
    ValidateQuery {
        /// The path to the file containing the query.
        path: Option<PathBuf>,
    },
}
