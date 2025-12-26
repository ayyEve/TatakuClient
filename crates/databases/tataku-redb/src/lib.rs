
mod scores;
mod wrapper;
mod beatmaps;
mod preferences;
use tataku_engine::*;

const DATABASE_PATH:&str = "tataku.redb";

pub struct Database(pub(crate) redb::Database);
impl Database {
    pub fn new() -> Self {
        let db = redb::Database::create(DATABASE_PATH)
            .expect("error opening database");

        Self(db)
    }
}

#[allow(clippy::needless_pass_by_value, reason = "used in map_err")]
impl Database {
    fn transaction_err(err: redb::TransactionError) -> tataku::Error {
        tataku::Error::String(err.to_string())
    }
    fn table_err(err: redb::TableError) -> tataku::Error {
        tataku::Error::String(err.to_string())
    }
    fn storage_err(err: redb::StorageError) -> tataku::Error {
        tataku::Error::String(err.to_string())
    }
}

impl engine::database::DatabaseProvider for Database {}
