use std::marker::PhantomData;

/// A connection to a G1 database.
pub struct Connection;

impl Connection {
    /// Opens a connection to the database at the given URL.
    pub async fn open(_url: &str) -> Result<Connection, OpenError> {
            unimplemented!()
    }

    /// Builds a query.
    pub fn build_query<F, T>(&self, func: F) -> Query<T>
        where F: FnOnce(QueryBuilder) -> Query<T>,
    {
        unimplemented!()
    }
}

pub struct OpenError;
pub struct QueryError;

pub struct Query<T>(PhantomData<T>);

impl<T> Query<T> {
    /// Runs the query, returning a single result.
    pub async fn run1(&self, conn: &Connection) -> Result<T, QueryError> {unimplemented!()}
}

pub struct QueryBuilder;
