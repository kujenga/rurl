extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate clap;

use std::borrow::Cow;
use std::str::FromStr;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::{Client, Request, Method};
use tokio_core::reactor::Core;
use clap::{Arg, App};

fn main() {
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
                .short("X")
                .long("method")
                .help("HTTP Method")
                .default_value("GET")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("HEADER")
                .short("H")
                .help("header")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    let uri_str = matches.value_of("URL").unwrap();
    let method = matches.value_of("METHOD").unwrap();
    let headers = matches.values_of("HEADER");

    match run(uri_str, method, headers) {
        Ok(_) => (),
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn run(
    uri_str: &str,
    method: &str,
    headers: std::option::Option<clap::Values>,
) -> std::result::Result<(), Box<std::error::Error>> {
    let mut core = Core::new()?;
    let client = Client::new(&core.handle());

    let uri = uri_str.parse()?;
    let method = Method::from_str(method).unwrap();
    let mut req = Request::new(method, uri);
    for val in headers.unwrap_or_default() {
        let h = ArbitraryHeader::from_str(val)?;
        req.headers_mut().set_raw(h.name, h.value);
    }

    let work = client.request(req).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    });

    Ok(core.run(work)?)
}

struct ArbitraryHeader {
    name: Cow<'static, str>,
    value: hyper::header::Raw,
}

impl std::str::FromStr for ArbitraryHeader {
    type Err = hyper::Error;
    fn from_str(s: &str) -> hyper::Result<ArbitraryHeader> {
        let idx = s.rfind(":").unwrap();
        let name = &s[..idx];
        let value = &s[idx + 1..];

        Ok(ArbitraryHeader {
            name: name.to_owned().into(),
            value: value.to_owned().into(),
        })
    }
}
