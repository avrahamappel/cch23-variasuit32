use std::fs::{self, File};
use std::io::Seek;
use std::path::Path;

use git2::{Commit, ObjectType, Repository, Sort, Tree};
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

fn open_repo(path: &Path) -> Result<Repository, Error> {
    let file = File::open(path)?;
    let mut arc = Archive::new(file);

    let dir = path.parent().unwrap_or(Path::new(""));
    let dir = dir.join("cch23-git-repo");
    fs::remove_dir_all(&dir)?;
    arc.unpack(&dir)?;

    let mut file = arc.into_inner();
    file.rewind()?;
    let mut arc = Archive::new(file);
    let first_entry = arc.entries()?.find_map(Result::ok).unwrap();
    let gitdir = first_entry.path()?.to_path_buf();

    let repo = Repository::open_bare(format!("{}/{}", dir.display(), gitdir.display()))?;
    Ok(repo)
}

fn filter_tree(tree: Tree, rev: &Commit, repo: &Repository) -> Option<String> {
    tree.iter()
        .filter_map(|leaf| match leaf.kind()? {
            ObjectType::Tree => {
                let Ok(tree) = leaf.to_object(repo).and_then(|obj| obj.peel_to_tree()) else {
                    return None;
                };

                filter_tree(tree, rev, repo)
            }
            ObjectType::Blob => {
                if !leaf.name()?.contains("santa.txt") {
                    return None;
                }

                if !leaf
                    .to_object(repo)
                    .and_then(|obj| obj.peel_to_blob())
                    .is_ok_and(|blob| String::from_utf8_lossy(blob.content()).contains("COOKIE"))
                {
                    return None;
                }

                let commit = format!("{} {}", rev.author().name().unwrap_or_default(), rev.id());

                Some(commit)
            }
            _ => None,
        })
        .take(1)
        .next()
}

fn find_commit(repo: Repository) -> Result<String, Error> {
    let mut revwalk = repo.revwalk()?;

    for refname in repo.references()?.names().filter_map(Result::ok) {
        revwalk.push_ref(refname)?;
    }

    revwalk.set_sorting(Sort::TIME)?;

    let commit = revwalk
        .filter_map(Result::ok)
        .filter_map(|oid| repo.find_commit(oid).ok())
        .find_map(|rev| {
            let Ok(tree) = rev.tree() else { return None };
            filter_tree(tree, &rev, &repo)
        })
        .unwrap_or_default();

    Ok(commit)
}

#[post("/cookie", data = "<tarball>")]
fn find_cookie(tarball: TempFile) -> Result<String, Error> {
    let path = tarball.path().expect("temp file had no path");
    let repo = open_repo(path)?;
    find_commit(repo)
}

pub fn routes() -> Vec<Route> {
    routes![tar_file_count, tar_files_size, find_cookie]
}
