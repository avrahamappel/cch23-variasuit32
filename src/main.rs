use std::ffi::OsStr;
use std::path::PathBuf;

use rocket::{get, routes, Responder};

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

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, fake_error, exclusive_cube]);

    Ok(rocket.into())
}
