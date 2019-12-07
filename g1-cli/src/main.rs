use anyhow::Result;
use directories::BaseDirs;
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
use std::{collections::BTreeSet, io::Read, path::PathBuf, sync::Arc, thread::spawn};
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
    let history_path = BaseDirs::new().map(|bd| bd.cache_dir().join("g1_repl_history"));
    spawn(move || {
        let r = (|| {
            let reader = Interface::new("g1-cli")?;
            reader.set_prompt("g1> ")?;
            if let Some(path) = history_path.as_ref() {
                if let Err(err) = reader.load_history(path) {
                    log::debug!("Failed to load history: {}", err);
                }
            }
            loop {
                block_on(recv_wait.recv());
                match reader.read_line()? {
                    ReadResult::Input(input) => {
                        reader.add_history_unique(input.clone());
                        if let Some(path) = history_path.as_ref() {
                            if let Err(err) = reader.save_history(path) {
                                log::debug!("Failed to save history: {}", err);
                            }
                        }
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
            Err(e) => println!("Error: {}", e),
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
        }
        Command::CreateAtom => {
            println!("{}", conn.create_atom().await?);
        }
        Command::DeleteAtom(atom) => {
            if conn.delete_atom(atom.parse()?).await? {
                println!("Deleted atom.");
            } else {
                println!("Atom did not exist.");
            }
        }
        Command::CreateName(atom, ns, title, upsert) => {
            if conn.create_name(atom.parse()?, &ns, &title, upsert).await? {
                println!("Updated name.");
            } else {
                println!("Created name.");
            }
        }
        Command::DeleteName(ns, title) => {
            if conn.delete_name(&ns, &title).await? {
                println!("Deleted name.");
            } else {
                println!("Name did not exist.");
            }
        }
        Command::CreateEdge(from, to, label) => {
            if conn.create_edge(from.parse()?, to.parse()?, &label).await? {
                println!("Edge already existed.");
            } else {
                println!("Created edge.");
            }
        }
        Command::DeleteEdge(from, to, label) => {
            if conn.delete_edge(from.parse()?, to.parse()?, &label).await? {
                println!("Deleted edge.");
            } else {
                println!("Edge did not exist.");
            }
        }
        Command::CreateTag(atom, key, value, upsert) => {
            if conn.create_tag(atom.parse()?, &key, &value, upsert).await? {
                println!("Updated tag.");
            } else {
                println!("Created tag.");
            }
        }
        Command::DeleteTag(atom, key) => {
            if conn.delete_tag(atom.parse()?, &key).await? {
                println!("Deleted tag.");
            } else {
                println!("Tag did not exist.");
            }
        }
        Command::CreateBlob(atom, kind, mime, hash, upsert) => {
            if conn
                .create_blob(atom.parse()?, &kind, mime.parse()?, hash.parse()?, upsert)
                .await?
            {
                println!("Updated blob.");
            } else {
                println!("Created blob.");
            }
        }
        Command::DeleteBlob(atom, kind, mime) => {
            if conn
                .delete_blob(atom.parse()?, &kind, mime.parse()?)
                .await?
            {
                println!("Deleted blob.");
            } else {
                println!("Blob did not exist.");
            }
        }
        Command::Help => {
            println!(".help    Prints this help message.");
            println!(".quit    Quits the REPL.");
            println!();
            println!(".list                  Lists the existing predicates.");
            println!("<CLAUSE>               Adds a clause to a predicate, possibly defining it.");
            println!("?- <QUERY>.            Performs a query.");
            println!(".undefine <FUNCTOR>    Undefines defined predicates with the given functor.");
            println!();
            println!(".create_atom           Creates a new atom in the database, printing it.");
            println!(
                ".delete_atom <ATOM>    Deletes any names referring to an atom, all edges going"
            );
            println!(
                "                       to or from it, any tags attached to it, and any blobs"
            );
            println!("                       attached to it.");
            println!();
            println!(".create_name <ATOM> <NS> <TITLE>    Creates a new name for an atom.");
            println!(".upsert_name <ATOM> <NS> <TITLE>    Upserts a new name for an atom.");
            println!(".delete_name <NS> <TITLE>           Deletes a name.");
            println!();
            println!(".create_edge <FROM> <TO> <LABEL>    Creates a new edge between two atoms.");
            println!(
                ".delete_edge <FROM> <TO> <LABEL>    Deletes the edge with the given endpoints"
            );
            println!("                                    and label.");
            println!();
            println!(
                ".create_tag <ATOM> <KEY> <VALUE>    Creates a tag attached to an atom with the"
            );
            println!("                                    given key and value.");
            println!(
                ".delete_tag <ATOM> <KEY>            Deletes the tag with the given key from the"
            );
            println!("                                    given atom.");
            println!();
            println!(
                ".create_blob <ATOM> <KIND> <MIME> <HASH>    Creates a blob attached to an atom"
            );
            println!(
                "                                            with the given kind, MIME type, and"
            );
            println!("                                            hash.");
            println!(".delete_blob <KIND> <MIME>                  Deletes the blob with the given");
            println!(
                "                                            kind and MIME type from the given"
            );
            println!("                                            atom.");
        }
        Command::List => {
            let mut functors = BTreeSet::new();
            for c in clauses.iter() {
                let _ = functors.insert((&c.head.name, c.head.args.len()));
            }
            println!("{} predicates defined:", functors.len());
            for (name, argn) in functors {
                println!("{}/{}", name, argn);
            }
        }
        Command::Query(goal) => {
            let query = NamelessQuery::from_query::<C::Error>(Query {
                clauses: clauses.clone(),
                goal,
            })?;
            let solns = conn.query(None, &query).await?;
            print_solns(&solns);
        }
        Command::Quit => return Ok(true),
        Command::Undefine(name, argn) => {
            clauses.retain(|c| c.head.name != name || c.head.args.len() != argn as usize)
        }
    }
    Ok(false)
}

fn print_solns(solns: &[Vec<Arc<str>>]) {
    println!("Got {} results:", solns.len());
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
