use anyhow::Result;
use g1_common::{naive_solve::naive_solve_selfcontained, nameless::NamelessQuery};
use std::{io::Read, path::PathBuf};

#[paw::main]
fn main(args: Args) -> Result<()> {
    femme::start(match args.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    })?;

    match args.subcommand {
        Subcommand::RunSelfContained { path } => {
            let query = load_query(path)?;
            let solns = naive_solve_selfcontained(&query);
            println!("Found {} solutions:", solns.len());
            for soln in solns {
                println!("{:?}", soln);
            }
            Ok(())
        }
        Subcommand::ValidateQuery { path } => {
            let _ = load_query(path)?;
            println!("Validated!");
            Ok(())
        }
    }
}

fn load_query(path: Option<PathBuf>) -> Result<NamelessQuery> {
    let src = match path {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut src = String::new();
            std::io::stdin().read_to_string(&mut src)?;
            src
        }
    };
    let query = NamelessQuery::from_str::<g1_common::SimpleError>(&src)?;
    Ok(query)
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
    /// Runs a query without access to the database.
    RunSelfContained {
        /// The path to the file containing the query.
        path: Option<PathBuf>,
    },

    /// Validates that a query is valid.
    ValidateQuery {
        /// The path to the file containing the query.
        path: Option<PathBuf>,
    },
}
