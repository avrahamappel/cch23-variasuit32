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

#[post("/archive_files_size", data = "<tarball>")]
fn tar_files_size(tarball: TempFile) -> Result<String, Error> {
    let mut arc = Archive::new(File::open(tarball.path().expect("temp file had no path"))?);

    let sum = arc
        .entries()?
        .filter_map(Result::ok)
        .map(|e| e.size())
        .sum::<u64>();

    Ok(sum.to_string())
}

pub fn routes() -> Vec<Route> {
    routes![tar_file_count, tar_files_size]
}
