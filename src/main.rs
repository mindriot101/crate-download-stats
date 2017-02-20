#[cfg(feature="fetch-remote")]
extern crate hyper;

#[cfg(feature="fetch-remote")]
extern crate hyper_native_tls;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
extern crate postgres;
extern crate dotenv;

use std::io::Read;

#[cfg(feature="fetch-remote")]
use hyper::Client;

#[cfg(feature="fetch-remote")]
use hyper::header::Accept;

#[cfg(feature="fetch-remote")]
use hyper::net::HttpsConnector;

#[cfg(feature="fetch-remote")]
use hyper_native_tls::NativeTlsClient;
use chrono::prelude::*;

use dotenv::dotenv;
use std::env;
use postgres::{Connection, TlsMode};

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug, Serialize, Deserialize)]
struct DownloadInfo {
    meta: Downloads,
    version_downloads: Vec<VersionDownload>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Downloads {
    extra_downloads: Vec<BasicDownload>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BasicDownload {
    date: NaiveDate,
    downloads: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionDownload {
    date: DateTime<UTC>,
    downloads: i64,
    id: i32,
    version: i32,
}

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

fn upload(info: Vec<VersionDownload>) -> Result<()> {
    let connection = establish_connection()?;
    reset_database(&connection)?;
    
    for entry in info {
        let trans = connection.transaction()?;
        trans.execute("INSERT INTO crate_downloads (
        id, date, downloads, version) VALUES ($1, $2, $3, $4)",
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
    let parsed: DownloadInfo = serde_json::from_str(&raw_response)?;
    upload(parsed.version_downloads)
}

#[cfg(feature="fetch-remote")]
fn fetch_raw_response(url: &str) -> Result<String> {
    let ssl = NativeTlsClient::new()?;
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let mut res = client.get(url)
        .header(Accept::json())
        .send()?;
    if res.status != hyper::Ok {
        return Err("Error sending client request".into());
    }

    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(body)
}

#[cfg(not(feature="fetch-remote"))]
fn fetch_raw_response(_url: &str) -> Result<String> {
    use std::fs::File;

    let mut f = File::open("testdata/test_response.txt")?;
    let mut body = String::new();
    f.read_to_string(&mut body)?;
    Ok(body)
}
