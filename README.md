g1
==

A simple graph store.

Model
-----

### Atom

Atoms are the nodes of the graph. Each is represented as a UUID.

### Names

Names uniquely identify an atom. They have a namespace and a title, both of which are strings.

### Edge

Edges are directed, with an atom at both endpoints. Edges have a string associated with them.

### Tag

Tags are attached to atoms. They have a kind and a value, both of which are strings.

### Blob

Blobs are attached to atoms. They have a type, which is a MIME type, and contents, which is an arbitrarily large binary string.

### Misc.

strings are UTF-8 strings, which should be no longer than 256 characters.

Rust API
--------

For now, only simple operations are allowed:

```rust
use anyhow::Result;
use g1::{Atom, Connection};

async fn example() -> Result<()> {
	let conn = Connection::open("http://g1.example.com/")?;
}
```
