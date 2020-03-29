extern crate corr_server_lib;
extern crate clap;
extern crate toml;
use corr_server_lib::bootstrap_server;
use clap::{App, Arg};
use std::fs::read_to_string;
use corr_server_lib::Config;

fn main() {
    let matches= App::new("Correlate Server")
        .version("0.0.2")
        .author("Atmaram R. Naik <naik.atmaram@gmail.com>")
        .about("Command line tool for api journey processor server")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .default_value("/usr/local/etc/corrs.toml")
            .help("Sets a custom config file")
            .takes_value(true)
        )
        .get_matches();

        let mut path = matches.value_of("config").unwrap();
        let config : Config = toml::from_str(read_to_string(path).unwrap().as_str()).unwrap();
        bootstrap_server(config);
}
