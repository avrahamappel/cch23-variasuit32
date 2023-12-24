use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{post, routes, Route};

#[derive(Serialize, Clone)]
#[serde(untagged, crate = "rocket::serde")]
enum Names<'n> {
    Single(&'n str),
    List(Vec<Names<'n>>),
}

#[post("/?<offset>&<limit>&<split>", data = "<names>")]
fn name_slice(
    names: Json<Vec<&str>>,
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
) -> Json<Names> {
    let off = offset.unwrap_or(0);
    let lim = limit.unwrap_or(names.len());

    let slice: Vec<_> = names
        .into_inner()
        .into_iter()
        .skip(off)
        .take(lim)
        .map(Names::Single)
        .collect();

    if let Some(spl) = split {
        let list: Vec<_> = slice
            .chunks(spl)
            .map(|cs| Names::List(cs.to_vec()))
            .collect();
        Json(Names::List(list))
    } else {
        Json(Names::List(slice))
    }
}

pub fn routes() -> Vec<Route> {
    routes![name_slice]
}
