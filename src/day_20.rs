use std::fs::File;
use std::io::Seek;
use std::path::Path;

use git2::{ObjectType, Repository, Sort};
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

#[post("/cookie", data = "<tarball>")]
fn find_cookie(tarball: TempFile) -> Result<String, Error> {
    let path = tarball.path().expect("temp file had no path");
    let file = File::open(path)?;
    let mut arc = Archive::new(file);

    let dir = path.parent().unwrap_or(Path::new(""));
    arc.unpack(dir)?;

    let mut file = arc.into_inner();
    file.rewind()?;
    let mut arc = Archive::new(file);
    let first_entry = arc.entries()?.find_map(Result::ok).unwrap();
    let gitdir = first_entry.path()?.to_path_buf();

    let repo = Repository::open_bare(format!("{}/{}", dir.display(), gitdir.display()))?;

    let mut revwalk = repo.revwalk()?;

    for refr in repo
        .references()?
        .filter_map(Result::ok)
        .filter_map(|r| r.name().map(str::to_string))
    {
        revwalk.push_ref(&refr)?;
    }

    revwalk.set_sorting(Sort::TIME)?;

    let commit = revwalk
        .filter_map(Result::ok)
        .filter_map(|oid| repo.find_commit(oid).ok())
        .find_map(|rev| {
            let Ok(tree) = rev.tree() else { return None };

            tree.iter()
                .filter(|leaf| matches!(leaf.kind(), Some(ObjectType::Blob)))
                .filter(|leaf| leaf.name().is_some_and(|name| name.contains("santa.txt")))
                .filter(|leaf| {
                    leaf.to_object(&repo)
                        .and_then(|obj| obj.peel_to_blob())
                        .is_ok_and(|blob| {
                            String::from_utf8_lossy(blob.content()).contains("COOKIE")
                        })
                })
                .map(move |_| format!("{} {}", rev.author().name().unwrap_or_default(), rev.id()))
                .take(1)
                .next()
        })
        .unwrap_or_default();

    Ok(commit)
}

pub fn routes() -> Vec<Route> {
    routes![tar_file_count, tar_files_size, find_cookie]
}
