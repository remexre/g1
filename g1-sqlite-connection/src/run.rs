use crate::{Atom, Command, SqliteError};
use g1_common::naive_solve::naive_solve;
use log::error;
use rusqlite::{Connection, NO_PARAMS};
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

impl Command {
    pub(crate) fn run(self, conn: &mut Connection) {
        match self {
            Command::CreateAtom(send) => {
                with_sender(send, || {
                    let mut retries = 3;
                    loop {
                        let atom = Atom::new();
                        match conn.execute("insert into atoms values (?)", &[atom.to_string()]) {
                            Ok(_) => break Ok(atom),
                            Err(rusqlite::Error::SqliteFailure(
                                rusqlite::ffi::Error {
                                    code: rusqlite::ErrorCode::ConstraintViolation,
                                    extended_code: 2067,
                                },
                                _,
                            )) if retries > 0 => {
                                retries -= 1;
                                error!("Failed to create atom; check your entropy")
                            }
                            Err(e) => break Err(e.into()),
                        }
                    }
                });
            }
            Command::DeleteAtom(_atom, _send) => error!("TODO"),
            Command::CreateName(_atom, _ns, _title, true, _send) => error!("TODO"),
            Command::CreateName(atom, ns, title, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into names values (?, ?, ?)",
                        &[atom.to_string(), ns, title],
                    )?;
                    Ok(false)
                });
            }
            Command::DeleteName(_ns, _title, _send) => error!("TODO"),
            Command::CreateEdge(from, to, label, send) => {
                with_sender(send, move || {
                    match conn.execute(
                        "insert into edges values (?, ?, ?)",
                        &[from.to_string(), to.to_string(), label],
                    ) {
                        Ok(_) => Ok(false),
                        Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error {
                                code: rusqlite::ErrorCode::ConstraintViolation,
                                extended_code: 2067,
                            },
                            _,
                        )) => Ok(true),
                        Err(e) => Err(e.into()),
                    }
                });
            }
            Command::DeleteEdge(_from, _to, _label, _send) => error!("TODO"),
            Command::CreateTag(_atom, _key, _value, true, _send) => error!("TODO"),
            Command::CreateTag(atom, key, value, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into tags values (?, ?, ?)",
                        &[atom.to_string(), key, value],
                    )?;
                    Ok(false)
                });
            }
            Command::DeleteTag(_atom, _key, _send) => error!("TODO"),
            Command::CreateBlob(_atom, _kind, _mime, _hash, true, _send) => error!("TODO"),
            Command::CreateBlob(atom, kind, mime, hash, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into blobs values (?, ?, ?, ?)",
                        &[atom.to_string(), kind, mime.to_string(), hash.to_string()],
                    )?;
                    Ok(false)
                });
            }
            Command::DeleteBlob(_atom, _kind, _mime, _send) => error!("TODO"),
            Command::Query(limit, query, send) => {
                with_sender(send, move || {
                    let tx = conn.transaction()?;

                    let atoms = tx
                        .prepare("select atom from atoms")?
                        .query_and_then(NO_PARAMS, |row| Ok(Arc::from(row.get::<_, String>(0)?)))?
                        .collect::<Result<Vec<_>, SqliteError>>()?;

                    let names = tx
                        .prepare("select atom, ns, title from names")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteError>>()?;

                    let edges = tx
                        .prepare("select edge_from, edge_to, label from edges")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteError>>()?;

                    let tags = tx
                        .prepare("select atom, key, value from tags")?
                        .query_and_then(NO_PARAMS, |row| {
                            Ok((
                                Arc::from(row.get::<_, String>(0)?),
                                Arc::from(row.get::<_, String>(1)?),
                                Arc::from(row.get::<_, String>(2)?),
                            ))
                        })?
                        .collect::<Result<Vec<_>, SqliteError>>()?;

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
                        .collect::<Result<Vec<_>, SqliteError>>()?;

                    tx.finish()?;

                    Ok(naive_solve(
                        &atoms, &names, &edges, &tags, &blobs, limit, &query,
                    ))
                });
            }
        }
    }
}

fn with_sender<F, T>(send: Sender<Result<T, SqliteError>>, func: F)
where
    F: FnOnce() -> Result<T, SqliteError>,
{
    let _ = send.send(func());
}
