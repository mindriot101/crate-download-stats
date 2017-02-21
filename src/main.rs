#[cfg(feature="fetch-remote")]
extern crate curl;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
extern crate postgres;
extern crate dotenv;
extern crate clap;

#[cfg(feature="fetch-remote")]
use curl::easy::Easy;
use chrono::prelude::*;

mod models;
mod database;
mod cli;

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



fn run() -> Result<()> {
    let matches = cli::cmdline_args();
    match matches.values_of("crate") {
        Some(crate_names) => {
            let now_timestamp = UTC::now().timestamp();
            for crate_name in crate_names {
                let url = &format!("https://crates.io/api/v1/crates/{}/downloads?_={}",
                                crate_name, now_timestamp);
                let raw_response = fetch_raw_response(url)?;
                let parsed: models::DownloadInfo = serde_json::from_str(&raw_response)?;
                database::upload(parsed.version_downloads, &crate_name)?;
            }
            Ok(())
        },
        None => Err("Could not parse crate name from the command line".into()),
    }
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
