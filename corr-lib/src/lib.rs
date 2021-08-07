#![feature(generators, generator_trait)]
#![feature(async_closure)]
extern crate lazy_static;
pub mod journey;
pub mod core;
pub mod template;
pub mod parser;
extern crate nom;
pub fn get_keywords<'a>()->Vec<&'a str>{
    let concatenated = [&get_journey_keywords()[..], &get_scriptlet_keywords()[..]].concat();
    return concatenated;
}
pub fn get_journey_keywords<'a>()->Vec<&'a str>{
    return vec!["print","respond","async","push","sync","load","object","text","for","let","request","url","body","headers","get","put","post","patch","delete","matching","and"]
}
pub fn get_scriptlet_keywords<'a>()->Vec<&'a str>{
    return vec!["null","true","false"]
}
