#[cfg(feature="fetch-remote")]
extern crate curl;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
extern crate postgres;
extern crate dotenv;

#[cfg(feature="fetch-remote")]
use curl::easy::Easy;
use chrono::prelude::*;
use dotenv::dotenv;
use std::env;
use postgres::{Connection, TlsMode};

mod models;

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        ::std::process::exit(1);
    }
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

fn upload(info: Vec<models::VersionDownload>) -> Result<()> {
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

fn run() -> Result<()> {
    let now_timestamp = UTC::now().timestamp();
    let url = &format!("https://crates.io/api/v1/crates/fitsio/downloads?_={}",
                       now_timestamp);
    let raw_response = fetch_raw_response(url)?;
    let parsed: models::DownloadInfo = serde_json::from_str(&raw_response)?;
    upload(parsed.version_downloads)
}

#[cfg(feature="fetch-remote")]
fn fetch_raw_response(url: &str) -> Result<String> {
    let mut easy = Easy::new();
    let mut dst = Vec::new();
    easy.url(url)?;

    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;

        transfer.perform()?;
    }

    let result = String::from_utf8(dst)?;
    Ok(result)
}

#[cfg(not(feature="fetch-remote"))]
fn fetch_raw_response(_url: &str) -> Result<String> {
    use std::fs::File;

    let mut f = File::open("testdata/test_response.txt")?;
    let mut body = String::new();
    f.read_to_string(&mut body)?;
    Ok(body)
}
