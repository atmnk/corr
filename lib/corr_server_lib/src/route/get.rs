use std::io;
use rocket::response::{NamedFile};
use rocket::State;
use crate::Config;
use std::path::{Path};

#[get("/")]
pub fn index(state:State<Config>) -> io::Result<NamedFile> {
    NamedFile::open(Path::new(state.wroot.as_str()).join("index.html"))
}



