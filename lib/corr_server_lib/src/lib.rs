#![feature(
proc_macro_hygiene,
decl_macro,
rustc_private,
type_ascription
)]
#[macro_use]
extern crate rocket;

use std::thread;
mod route;
mod processor;
use crate::route::{get, static_files};
fn rocket() -> rocket::Rocket {
    let rocket_routes = routes![
        static_files::file,
        get::index
        ];

    rocket::ignite()
        .mount("/", rocket_routes)
}
pub fn bootstrap_server() {
    thread::spawn(||{
        processor::create_server();
    });
    rocket().launch();
}

