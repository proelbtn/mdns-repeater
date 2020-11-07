#[macro_use]
extern crate clap;

use clap::{App, Arg};

const ARGS_INTERFACES: &'static str = "INTERFACES";

fn setup_app<'a, 'b>() -> App<'a, 'b> {
    let name = env!("CARGO_PKG_NAME");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let version = crate_version!();
    let author = env!("CARGO_PKG_AUTHORS");
    App::new(name)
        .version(version)
        .author(author)
        .about(description)
        .arg(Arg::with_name(ARGS_INTERFACES)
            .help("Interface names where mdns-repeater works")
            .required(true)
            .multiple(true))
}

fn main() {
    let matches = setup_app().get_matches();
    let interfaces = matches.values_of(ARGS_INTERFACES).unwrap();
    for interface in interfaces {
        println!("{:?}", interface);
    }
}
