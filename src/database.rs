use super::models;
use super::Result;
use postgres::{Connection, TlsMode};
use dotenv::dotenv;
use std::env;

pub fn upload(info: Vec<models::VersionDownload>) -> Result<()> {
    let connection = establish_connection()?;
    reset_database(&connection)?;

    for entry in info {
        let trans = connection.transaction()?;
        trans.execute("INSERT INTO crate_downloads (
        id, date, downloads, version) VALUES \
                      ($1, $2, $3, $4)",
                     &[&entry.id, &entry.date, &entry.downloads, &entry.version])
            .unwrap_or(0);
        trans.commit().unwrap_or(());
    }

    Ok(())
}

fn establish_connection() -> Result<Connection> {
    dotenv().ok();
    let database_uri = env::var("DATABASE_URL")?;
    let connection = Connection::connect(database_uri, TlsMode::None)?;
    Ok(connection)
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS crate_downloads (
        id SERIAL PRIMARY KEY,
        date \
                  TIMESTAMP WITH TIME ZONE NOT NULL,
        downloads BIGINT NOT NULL,
        \
                  version INTEGER NOT NULL
)",
                 &[])?;
    Ok(())
}


#[cfg(feature = "reset-database")]
fn reset_database(conn: &Connection) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS crate_downloads", &[])?;
    create_table(conn)?;
    Ok(())
}

#[cfg(not(feature = "reset-database"))]
fn reset_database(conn: &Connection) -> Result<()> {
    create_table(conn)?;
    Ok(())
}
