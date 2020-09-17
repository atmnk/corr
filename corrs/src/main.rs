#![feature(generators, generator_trait)]
use corrs::server::Server;
use clap::Clap;
use std::fs::read_to_string;
use corrs::Config;

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let config:Config = toml::from_str(read_to_string(opts.config).unwrap().as_str()).unwrap();
    let server = Server::new(opts.port);
    println!("Running corrs on {}",opts.port);
    server.run(config).await;
}
#[derive(Clap,Debug)]
#[clap(version = "0.1.0", author = "Atmaram Naik <atmnk@yahoo.com>")]
struct Opts {
    #[clap(short, long, default_value = "8765")]
    port:u16,

    #[clap(short, long, default_value = "/usr/local/etc/corrs-cfg.toml")]
    config:String
}

