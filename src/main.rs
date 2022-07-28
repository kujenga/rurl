extern crate clap;
extern crate http;
extern crate reqwest;

use clap::{App, Arg};
use core::result::Result;
use http::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("rurl")
        .about("A simple alternative to curl, written in Rust.")
        .arg(
            Arg::with_name("URL")
                .help("URL to make a request to.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("METHOD")
                .short('X')
                .long("method")
                .help("HTTP Method")
                .default_value("GET")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("HEADER")
                .short('H')
                .help("header")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    let uri_str = matches.value_of("URL").unwrap();
    let method = matches.value_of("METHOD").unwrap();
    let headers: Option<Vec<&str>> = match matches.get_many::<String>("HEADER") {
        None => None,
        Some(h) => Some(h.map(|s| s.as_str()).collect()),
    };

    run(uri_str, method, headers).await
}

async fn run(
    uri_str: &str,
    method: &str,
    header_inputs: Option<Vec<&str>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let method = Method::from_str(method).unwrap();
    let mut headers = HeaderMap::new();
    for val in header_inputs.unwrap_or_default() {
        let h = ArbitraryHeader::from_str(val)?;
        headers.insert(h.name, h.value);
    }

    let req = reqwest::Client::new()
        .request(method, uri_str)
        .headers(headers);
    let resp = req.send().await?.text().await?;
    println!("{}", resp);
    Ok(())
}

// Provides an encapsulation of headers parsed out of curl-style CLI args.
struct ArbitraryHeader {
    name: HeaderName,
    value: HeaderValue,
}

impl std::str::FromStr for ArbitraryHeader {
    type Err = ArbitraryHeaderError;

    fn from_str(s: &str) -> Result<ArbitraryHeader, ArbitraryHeaderError> {
        let idx = s.rfind(":").unwrap();
        let name_str = &s[..idx];
        let value_str = &s[idx + 1..];

        Ok(ArbitraryHeader {
            name: HeaderName::from_bytes(name_str.as_bytes())?,
            value: HeaderValue::from_str(value_str)?,
        })
    }
}

// Provides an enumeration of possible error behaviors at parsing time when working to interpret
// curl-style headers into values that are passed into the request builder.
#[derive(Debug)]
enum ArbitraryHeaderError {
    Name(InvalidHeaderName),
    Value(InvalidHeaderValue),
}

impl From<InvalidHeaderName> for ArbitraryHeaderError {
    fn from(err: InvalidHeaderName) -> Self {
        ArbitraryHeaderError::Name(err)
    }
}

impl From<InvalidHeaderValue> for ArbitraryHeaderError {
    fn from(err: InvalidHeaderValue) -> Self {
        ArbitraryHeaderError::Value(err)
    }
}

impl std::fmt::Display for ArbitraryHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitraryHeaderError::Name(e) => write!(f, "{}", e),
            ArbitraryHeaderError::Value(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ArbitraryHeaderError {}
