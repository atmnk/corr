#![feature(generators, generator_trait)]
#![feature(async_closure)]
#![feature(test)]
extern crate lazy_static;
extern crate test;
extern crate influxdb2;
extern crate rand;
pub mod journey;
pub mod core;
pub mod template;
pub mod parser;
pub mod workload;
extern crate nom;
pub fn get_keywords<'a>()->Vec<&'a str>{
    let concatenated = [&get_journey_keywords()[..], &get_scriptlet_keywords()[..]].concat();
    return concatenated;
}
pub fn get_journey_keywords<'a>()->Vec<&'a str>{
    return vec![
        "close",
        "startup",
        "while",
        "undef",
        "measure",
        "ingest",
        "as",
        "wait",
        "exit",
        "print",
                "respond",
                "connect",
                "async",
                "call",
                "send",
                "websocket",
                "named",
                "server",
                "background",
                "connect",
                "listener",
                "postgres","form","push","sync","load","object","text","for","let","request","url","body","headers","get","put","post","patch","delete","matching","and"]
}
pub fn get_scriptlet_keywords<'a>()->Vec<&'a str>{
    return vec!["null","true","false"]
}
