extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate clap;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
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
        .get_matches();

    let uri_str = matches.value_of("URL").unwrap();

    match run(uri_str) {
        Ok(_) => (),
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn run(uri_str: &str) -> std::result::Result<(), Box<std::error::Error>> {
    let mut core = Core::new()?;
    let client = Client::new(&core.handle());
    let uri = uri_str.parse()?;
    let work = client.get(uri).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    });
    Ok(core.run(work)?)
}
