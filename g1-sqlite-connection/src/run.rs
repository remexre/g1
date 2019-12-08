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
            Command::DeleteAtom(atom, send) => with_sender(send, move || {
                let tx = conn.transaction()?;

                let _ = tx.execute("delete from names where atom = ?", &[atom.to_string()])?;
                let _ = tx.execute("delete from edges where edge_from = ?", &[atom.to_string()])?;
                let _ = tx.execute("delete from edges where edge_to = ?", &[atom.to_string()])?;
                let _ = tx.execute("delete from tags where atom = ?", &[atom.to_string()])?;
                let _ = tx.execute("delete from blobs where atom = ?", &[atom.to_string()])?;

                tx.finish()?;
                Ok(())
            }),
            Command::CreateName(atom, ns, title, true, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert or replace into names values (?, ?, ?)",
                        &[atom.to_string(), ns, title],
                    )?;
                    Ok(())
                });
            }
            Command::CreateName(atom, ns, title, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into names values (?, ?, ?)",
                        &[atom.to_string(), ns, title],
                    )?;
                    Ok(())
                });
            }
            Command::DeleteName(ns, title, send) => with_sender(send, move || {
                conn.execute("delete from names where ns = ? and title = ?", &[ns, title])
                    .map(|n| match n {
                        0 => false,
                        1 => true,
                        n => {
                            error!("unexpected result from deleting name: {}", n);
                            true
                        }
                    })
                    .map_err(From::from)
            }),
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
            Command::DeleteEdge(from, to, label, send) => with_sender(send, move || {
                conn.execute(
                    "delete from edges where edge_from = ? and edge_to = ? and label = ?",
                    &[from.to_string(), to.to_string(), label],
                )
                .map(|n| match n {
                    0 => false,
                    1 => true,
                    n => {
                        error!("unexpected result from deleting edge: {}", n);
                        true
                    }
                })
                .map_err(From::from)
            }),
            Command::CreateTag(atom, key, value, true, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert or replace into tags values (?, ?, ?)",
                        &[atom.to_string(), key, value],
                    )?;
                    Ok(())
                });
            }
            Command::CreateTag(atom, key, value, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into tags values (?, ?, ?)",
                        &[atom.to_string(), key, value],
                    )?;
                    Ok(())
                });
            }
            Command::DeleteTag(atom, key, send) => with_sender(send, move || {
                conn.execute(
                    "delete from tags where atom = ? and key = ?",
                    &[atom.to_string(), key],
                )
                .map(|n| match n {
                    0 => false,
                    1 => true,
                    n => {
                        error!("unexpected result from deleting tag: {}", n);
                        true
                    }
                })
                .map_err(From::from)
            }),
            Command::CreateBlob(atom, kind, mime, hash, true, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert or replace into blobs values (?, ?, ?, ?)",
                        &[atom.to_string(), kind, mime.to_string(), hash.to_string()],
                    )?;
                    Ok(())
                });
            }
            Command::CreateBlob(atom, kind, mime, hash, false, send) => {
                with_sender(send, move || {
                    let _ = conn.execute(
                        "insert into blobs values (?, ?, ?, ?)",
                        &[atom.to_string(), kind, mime.to_string(), hash.to_string()],
                    )?;
                    Ok(())
                });
            }
            Command::DeleteBlob(atom, kind, mime, send) => with_sender(send, move || {
                conn.execute(
                    "delete from blobs where atom = ? and kind = ? and mime = ?",
                    &[atom.to_string(), kind, mime.to_string()],
                )
                .map(|n| match n {
                    0 => false,
                    1 => true,
                    n => {
                        error!("unexpected result from deleting blob: {}", n);
                        true
                    }
                })
                .map_err(From::from)
            }),
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
