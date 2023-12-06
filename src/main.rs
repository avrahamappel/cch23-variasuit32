#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]

use std::ffi::OsStr;
use std::path::PathBuf;

use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, routes, Responder};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder)]
#[response(status = 500)]
struct Error {
    message: &'static str,
}

#[get("/-1/error")]
fn fake_error() -> Error {
    Error {
        message: "This is an error",
    }
}

#[get("/1/<nums..>")]
fn exclusive_cube(nums: PathBuf) -> String {
    nums.iter()
        .filter_map(OsStr::to_str)
        .filter_map(|s| s.parse::<u32>().ok())
        .fold(0, |acc, num| acc ^ num)
        .pow(3)
        .to_string()
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Reindeer {
    // name: &'n str,
    strength: u32,
}

#[post("/4/strength", data = "<reindeers>")]
fn reindeer_cheer(reindeers: Json<Vec<Reindeer>>) -> String {
    reindeers
        .iter()
        .map(|r| r.strength)
        .sum::<u32>()
        .to_string()
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount(
        "/",
        routes![index, fake_error, exclusive_cube, reindeer_cheer],
    );

    Ok(rocket.into())
}
