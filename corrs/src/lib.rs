#![feature(generators, generator_trait)]
extern crate lazy_static;
pub mod server;
pub mod hub;
use serde::{Deserialize};
#[derive(Deserialize)]
pub struct Config {
    wroot: String,
}