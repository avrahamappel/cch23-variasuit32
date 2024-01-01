use std::fs::File;

use rocket::fs::TempFile;
use rocket::{post, routes, Route};
use tar::Archive;

use crate::common::Error;

#[post("/archive_files", data = "<tarball>")]
fn tar_file_count(tarball: TempFile) -> Result<String, Error> {
    let mut arc = Archive::new(File::open(tarball.path().expect("temp file had no path"))?);

    let count = arc.entries()?.filter_map(Result::ok).count();

    Ok(count.to_string())
}

pub fn routes() -> Vec<Route> {
    routes![tar_file_count]
}
