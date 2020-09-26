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
    return vec!["print","fillable","text","for","let"]
}
pub fn get_scriptlet_keywords<'a>()->Vec<&'a str>{
    return vec!["add","sub","mul","div","concat","null","true","false"]
}
