extern crate clap;
extern crate http;
extern crate reqwest;

use clap::Parser;
use core::result::Result;
use http::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue};
use std::str::FromStr;

/// A simple alternative to curl, written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to make a request to
    #[arg(index = 1, required = true)]
    url: String,

    /// HTTP Method
    #[arg(short = 'X', long = "method")]
    method: Method,

    /// HTTP Header
    #[arg(short = 'H', long = "header")]
    header: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    run(args.url, args.method, args.header).await
}

async fn run(
    uri_str: String,
    method: Method,
    header_inputs: Option<Vec<String>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    for val in header_inputs.unwrap_or_default() {
        let h = ArbitraryHeader::from_str(&val)?;
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
#[derive(Debug, PartialEq)]
struct ArbitraryHeader {
    name: HeaderName,
    value: HeaderValue,
}

impl std::str::FromStr for ArbitraryHeader {
    type Err = ArbitraryHeaderError;

    fn from_str(s: &str) -> Result<ArbitraryHeader, ArbitraryHeaderError> {
        let idx = s.rfind(":").unwrap();
        let name_str = &s[..idx].trim();
        let value_str = &s[idx + 1..].trim();

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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_basic_header_parse() {
        assert_eq!(
            ArbitraryHeader::from_str("test: value").unwrap(),
            ArbitraryHeader {
                name: HeaderName::from_static("test"),
                value: HeaderValue::from_static("value"),
            }
        );
    }
}
