#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]

use std::ffi::OsStr;
use std::ops::Deref;
use std::path::PathBuf;

use base64::engine::general_purpose;
use base64::Engine;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, post, routes, Request, Responder};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder, Debug)]
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
struct Reindeer<'r> {
    name: &'r str,
    strength: u32,
    speed: f64,
    height: u32,
    antler_width: u32,
    snow_magic_power: u32,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies_eaten: u32,
    favorite_food: &'r str,
}

#[post("/4/strength", data = "<reindeers>")]
fn reindeer_cheer(reindeers: Json<Vec<Reindeer<'_>>>) -> String {
    reindeers
        .iter()
        .map(|r| r.strength)
        .sum::<u32>()
        .to_string()
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Winners {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

impl From<&[Reindeer<'_>]> for Winners {
    fn from(reindeers: &[Reindeer<'_>]) -> Self {
        let fastest = reindeers
            .iter()
            .max_by(|a, b| a.speed.total_cmp(&b.speed))
            .unwrap();
        let tallest = reindeers.iter().max_by_key(|r| r.height).unwrap();
        let magician = reindeers.iter().max_by_key(|r| r.snow_magic_power).unwrap();
        let consumer = reindeers.iter().max_by_key(|r| r.candies_eaten).unwrap();

        Self {
            fastest: format!(
                "Speeding past the finish line with a strength of {} is {}",
                fastest.strength, fastest.name
            ),
            tallest: format!(
                "{} is standing tall with his {} cm wide antlers",
                tallest.name, tallest.antler_width
            ),
            magician: format!(
                "{} could blast you away with a snow magic power of {}",
                magician.name, magician.snow_magic_power
            ),
            consumer: format!(
                "{} ate lots of candies, but also some {}",
                consumer.name, consumer.favorite_food
            ),
        }
    }
}

#[post("/4/contest", data = "<reindeers>")]
fn reindeer_candy(reindeers: Json<Vec<Reindeer<'_>>>) -> Json<Winners> {
    Json(Winners::from(reindeers.deref().as_slice()))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ElfCount {
    elf: usize,
    #[serde(rename = "elf on a shelf")]
    on_shelf: usize,
    #[serde(rename = "shelf with no elf on it")]
    shelf_no_elf: usize,
}

impl From<String> for ElfCount {
    fn from(elfstring: String) -> Self {
        let elf = elfstring.matches("elf").count();
        let on_shelf = elfstring.matches("elf on a shelf").count();
        let shelf_no_elf = elfstring
            .match_indices("shelf")
            .filter(|(i, _)| {
                let prefix = "elf on a ";
                &elfstring[(i - prefix.len())..*i] != prefix
            })
            .count();

        Self {
            elf,
            on_shelf,
            shelf_no_elf,
        }
    }
}

#[post("/6", data = "<elfstring>")]
fn elf_count(elfstring: String) -> Json<ElfCount> {
    Json(ElfCount::from(elfstring))
}

struct CookieHeader {
    value: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CookieHeader {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let error_outcome = || {
            Outcome::Error((
                Status::BadRequest,
                Error {
                    message: "Missing or invalid `Cookie` header",
                },
            ))
        };

        if let Some(h) = req.headers().get_one("Cookie") {
            if &h[0..7] != "recipe=" {
                return error_outcome();
            }

            let recipe = &h[7..];

            if let Ok(bytes) = general_purpose::STANDARD.decode(recipe) {
                let value = String::from_utf8_lossy(&bytes).into_owned();

                return Outcome::Success(CookieHeader { value });
            }
        }

        error_outcome()
    }
}

#[get("/7/decode")]
fn cookie_recipe(cookie_header: CookieHeader) -> String {
    cookie_header.value
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount(
        "/",
        routes![
            index,
            fake_error,
            exclusive_cube,
            reindeer_cheer,
            reindeer_candy,
            elf_count,
            cookie_recipe
        ],
    );

    Ok(rocket.into())
}
