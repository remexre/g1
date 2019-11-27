//! Keep this example synced with the `README.md`.

use anyhow::Result;
use g1::{Atom, Connection};

#[tokio::main]
async fn main() -> Result<()> {
    let conn = Connection::open("http://localhost:61616/")?;
    unimplemented!("{:?}", conn)
}
