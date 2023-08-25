pub mod auth;
pub mod pg;
pub mod redis;
pub mod schema;

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error("DieselError: {:?}", _0)]
    Diesel(#[from] diesel::result::Error),
    #[error("PooledConnectionError: cannot get the connection from r2d2 pool")]
    PooledConnection(String),
}
