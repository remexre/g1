use crate::{Command, SqliteConnectionError};
use g1_common::naive_solve::naive_solve;
use rusqlite::{Connection, NO_PARAMS};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

impl Command {
    pub(crate) fn run(self, conn: &mut Connection) {
        match self {
            Command::Query(limit, query, send) => {
                with_sender(send, move || {
                    let tx = conn.transaction()?;

                    let atoms = tx
                        .prepare("select atom from atoms")?
                        .query_and_then(NO_PARAMS, |row| Ok(Arc::from(row.get::<_, String>(0)?)))?
                        .collect::<Result<Vec<_>, SqliteConnectionError>>()?;

                    let names = tx
                        .prepare("select atom, ns, title from names")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteConnectionError>>()?;

                    let edges = tx
                        .prepare("select edge_from, edge_to, label from edges")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteConnectionError>>()?;

                    let tags = tx
                        .prepare("select atom, key, value from tags")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteConnectionError>>()?;

                    let blobs = tx
                        .prepare("select atom, kind, mime, hash from blobs")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                                Arc::from(row.get::<_, String>(3)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteConnectionError>>()?;

                    tx.finish()?;

                    Ok(naive_solve(
                        &atoms, &names, &edges, &tags, &blobs, limit, &query,
                    ))
                });
            }
            cmd => {
                eprintln!("TODO: {:?}", cmd);
            }
        }
    }
}

fn with_sender<F, T>(send: Sender<Result<T, SqliteConnectionError>>, func: F)
where
    F: FnOnce() -> Result<T, SqliteConnectionError>,
{
    let _ = send.send(func());
}
