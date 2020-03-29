use std::path::{Path, PathBuf};
use rocket::response::NamedFile;
use rocket::State;
use crate::Config;

#[get("/static/<file..>")]
pub fn file(file: PathBuf, state: State<Config>) -> Option<NamedFile> {
    NamedFile::open(Path::new(state.wroot.as_str()).join(file)).ok()
}

