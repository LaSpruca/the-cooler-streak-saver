use diesel::Connection;
use std::env;
use tracing::{error, info};

pub mod models;
pub mod schema;

embed_migrations!("migrations");

#[cfg(feature = "sqlite")]
pub use diesel::SqliteConnection as DbConnection;

#[cfg(feature = "mysql")]
pub use diesel::MysqlConnection as DbConnection;

#[cfg(feature = "postgres")]
pub use diesel::PgConnection as DbConnection;

pub fn create_connection() -> Option<DbConnection> {
    #[cfg(feature = "sqlite")]
    info!("Creating SQLite connection");
    #[cfg(feature = "mysql")]
    info!("Creating MySQL connection");
    #[cfg(feature = "postgres")]
    info!("Creating Postgresql connection");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    match DbConnection::establish(&database_url) {
        Ok(val) => Some(val),
        Err(ex) => {
            error!("Error connecting to database {ex}");
            None
        }
    }
}

pub fn run_migrations(
    connection: &DbConnection,
) -> Result<(), diesel_migrations::RunMigrationsError> {
    embedded_migrations::run(connection)
}
