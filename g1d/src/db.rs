use anyhow::Result;
use mime::Mime;
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::HashSet,
    path::Path,
    sync::{mpsc, Arc},
    thread::{spawn, JoinHandle},
};
use tokio::sync::oneshot;
use uuid::Uuid;

macro_rules! queries {
    ($($(#[$meta:meta])* fn $name:ident(&$self:ident $(,$arg:ident : $argt:ty)*) -> $outt:ty $body:block)*) => {
        #[allow(non_camel_case_types)]
        enum Query {
            $($name($($argt,)* oneshot::Sender<Result<$outt>>),)*
        }

        impl Database {
            $(
                $(#[$meta])*
                pub async fn $name(&mut self, $($arg : $argt),*) -> Result<$outt> {
                    let (send, recv) = oneshot::channel();
                    let query = Query::$name($($arg,)* send);
                    self.send.send(query)?;
                    recv.await?
                }
            )*
        }

        trait ConnectionExt {
            fn handle_query(&self, query: Query);

            $(fn $name(&self, $($arg : $argt),*) -> Result<$outt>;)*
        }

        impl ConnectionExt for Connection {
            fn handle_query(&self, query: Query) {
                match query {
                    $(Query::$name($($arg,)* send) => {
                        drop(send.send(self.$name($($arg),*)));
                    },)*
                }
            }

            $(fn $name(&$self, $($arg : $argt),*) -> Result<$outt> {
                $body
            })*
        }
    };
}

/// A connection to the database. Cheaply clonable.
///
/// Since database operations are synchronous, holds open a separate thread for them.
#[derive(Clone, Debug)]
pub struct Database {
    send: mpsc::SyncSender<Query>,
    thread: Arc<JoinHandle<()>>,
}

impl Database {
    /// Opens a database "connection" to the given path. Note that this blocks the current
    /// thread.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Database> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            create table if not exists names
              ( atom text not null
              , namespace text not null
              , title text not null
              , constraint nameUnique unique (namespace, title)
              );
            create table if not exists edges
              ( from text not null
              , to text not null
              , key text not null
              , constraint edgeUnique unique (from, to, key)
              );
            create table if not exists tags
              ( atom text not null
              , kind text not null
              , value text not null
              , constraint tagUnique unique (atom, kind)
              );
            create table if not exists blobs
              ( atom text not null
              , mime text not null
              , hash text not null
              , contents blob not null
              , constraint blobUnique unique (atom, mime, hash) -- checking contents is slow
              );
            "#,
        )?;
        let (send, recv) = mpsc::sync_channel(8);
        let thread = Arc::new(spawn(move || {
            recv.into_iter().for_each(|query| conn.handle_query(query));
        }));
        Ok(Database { send, thread })
    }
}

queries! {
    /*
    /// Gets a room by ID.
    fn get_room(&self, backend: String, id: RoomID) -> Option<Room> {
        self.query_row(
            r#"select (parent, name, sendable) from rooms
               where backend = ? and id = ? and deleted = 0"#,
            params![backend, id.0.clone()],
            |row| Ok(Room {
                id,
                parent: row.get::<_, Option<_>>(0)?.map(RoomID),
                name: row.get(1)?,
                sendable: row.get(2)?,
            }),
        ).optional().map_err(Into::into)
    }

    /// Inserts or updates a room.
    fn upsert_room(&self, _backend: String, _room: Room) -> () { unimplemented!() }

    /// Marks a room as deleted.
    fn delete_room(&self, _backend: String, _id: RoomID) -> () { unimplemented!() }

    /// Lists rooms.
    fn list_rooms(&self, backend: String) -> HashSet<RoomID> {
        let mut stmt = self.prepare(r"select id from rooms where backend = ?")?;
        let mut rows = stmt.query(params![backend])?;

        let mut rooms = HashSet::new();
        while let Some(row) = rows.next()? {
            let room = row.get(0)?;
            drop(rooms.insert(RoomID(room)));
        }

        Ok(rooms)
    }

    /// Gets a message by ID.
    fn get_message(&self, _backend: String, _id: MessageID) -> Option<Message> { unimplemented!() }

    /// Inserts or updates a message.
    fn upsert_message(&self, _backend: String, _message: Message) -> () { unimplemented!() }

    /// Marks a message as deleted.
    fn delete_message(&self, _backend: String, _id: MessageID) -> () { unimplemented!() }

    /// Lists messages.
    fn list_messages(
        &self,
        _backend: String,
        _room: RoomID,
        _before: Option<DateTime<Utc>>,
        _after: Option<DateTime<Utc>>
    ) -> HashSet<MessageID> { unimplemented!() }

    /// Lists the backends that have been seen.
    fn list_backends(&self) -> HashSet<String> {
        let mut stmt = self.prepare(r#"
            select distinct backend from rooms
            union
            select distinct backend from messages
        "#)?;
        let mut rows = stmt.query(params![])?;

        let mut backends = HashSet::new();
        while let Some(row) = rows.next()? {
            let backend = row.get(0)?;
            drop(backends.insert(backend));
        }

        Ok(backends)
    }

    /// Gets an attachment by hash.
    fn get_attachment(&self, _hash: String) -> Vec<u8> { unimplemented!() }
    */
}
