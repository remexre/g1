use anyhow::Result;
use futures::executor::block_on;
use g1_common::{
    command::Command,
    naive_solve::naive_solve_selfcontained,
    nameless::NamelessQuery,
    query::{Clause, Query},
    Connection,
};
use g1_sqlite_connection::SqliteConnection;
use linefeed::{Interface, ReadResult};
use std::{io::Read, path::PathBuf, sync::Arc, thread::spawn};
use tokio::sync::mpsc;

#[paw::main]
fn main(args: Args) -> Result<()> {
    femme::start(match args.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    })?;

    match args.subcommand {
        Subcommand::ReplSqlite { db_dir } => tokio::runtime::Builder::new()
            .enable_all()
            .threaded_scheduler()
            .build()?
            .block_on(async move {
                let conn = SqliteConnection::open(db_dir).await?;
                repl(conn).await
            }),
        Subcommand::RunSqlite { db_dir, query_path } => {
            let query = load_query(query_path)?;
            let solns = tokio::runtime::Builder::new()
                .enable_all()
                .threaded_scheduler()
                .build()?
                .block_on(async move {
                    let conn = SqliteConnection::open(db_dir).await?;
                    conn.query(None, &query).await
                })?;
            print_solns(&solns);
            Ok(())
        }
        Subcommand::RunSelfContained { path } => {
            let query = load_query(path)?;
            let solns = naive_solve_selfcontained(&query);
            print_solns(&solns);
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

async fn repl<C: Connection>(conn: C) -> Result<()> {
    // We spawn a thread for stdin, unfortunately.
    let (mut send_wait, mut recv_wait) = mpsc::channel::<()>(1);
    let (mut send_line, mut recv_line) = mpsc::channel::<Result<String>>(1);
    spawn(move || {
        let r = (|| {
            let reader = Interface::new("g1-cli")?;
            reader.set_prompt("g1> ")?;
            loop {
                block_on(recv_wait.recv());
                match reader.read_line()? {
                    ReadResult::Input(input) => {
                        reader.add_history_unique(input.clone());
                        block_on(send_line.send(Ok(input)))?
                    }
                    _ => break,
                }
            }
            Ok(())
        })();
        if let Err(e) = r {
            let _ = block_on(send_line.send(Err(e)));
        }
    });

    let mut clauses = Vec::new();
    loop {
        send_wait.send(()).await?;
        let line = recv_line.recv().await;
        let line = match line {
            Some(r) => r?,
            None => break,
        };

        match repl_one(line, &mut clauses, &conn).await {
            Ok(true) => break,
            Ok(false) => {}
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}

async fn repl_one<C: Connection>(
    line: String,
    clauses: &mut Vec<Clause>,
    conn: &C,
) -> Result<bool> {
    match line.parse()? {
        Command::Clause(clause) => {
            clauses.push(clause);
            Ok(false)
        }
        Command::Query(goal) => {
            let query = NamelessQuery::from_query::<C::Error>(Query {
                clauses: clauses.clone(),
                goal,
            })?;
            let solns = conn.query(None, &query).await?;
            print_solns(&solns);
            Ok(false)
        }
        Command::Quit => Ok(true),
        Command::Undefine(name, argn) => {
            clauses.retain(|c| c.head.name != name || c.head.args.len() != argn as usize);
            Ok(false)
        }
        cmd => {
            dbg!(cmd);
            Ok(false)
        }
    }
}

fn print_solns(solns: &[Vec<Arc<str>>]) {
    println!("Found {} solutions:", solns.len());
    for soln in solns {
        print!("(");
        let mut first = true;
        for s in soln {
            if first {
                first = false;
            } else {
                print!(", ");
            }
            print!("{:?}", s);
        }
        println!(")");
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
    /// Runs a REPL using an SQLite connection.
    ReplSqlite {
        /// The path to the directory containing the SQLite database and blobs.
        #[structopt(short = "D", long = "db")]
        db_dir: PathBuf,
    },

    /// Runs a query using an SQLite connection.
    RunSqlite {
        /// The path to the directory containing the SQLite database and blobs.
        #[structopt(short = "D", long = "db")]
        db_dir: PathBuf,

        /// The path to the file containing the query.
        query_path: Option<PathBuf>,
    },

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
