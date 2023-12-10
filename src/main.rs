#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]

use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::Deref;
use std::path::PathBuf;

use base64::engine::general_purpose;
use base64::Engine;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::{serde_json, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, post, routes, Request, Responder};
use rustemon::client::RustemonClient;
use rustemon::pokemon::pokemon;

#[cfg(test)]
macro_rules! test_client {
    () => {
        rocket::local::blocking::Client::tracked(rocket()).unwrap()
    };
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
#[test]
fn index_test() {
    let client = test_client!();
    let response = client.get("/").dispatch();
    assert_eq!(Status::Ok, response.status());
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

#[cfg(test)]
#[test]
fn fake_error_test() {
    let client = test_client!();
    let response = client.get("/-1/error").dispatch();
    assert_eq!(Status::InternalServerError, response.status());
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

#[cfg(test)]
#[test]
fn exclusive_cube_test() {
    for (expected, url) in [("1728", "/1/4/8"), ("1000", "/1/10"), ("27", "/1/4/5/8/10")] {
        let client = test_client!();
        let response = client.get(url).dispatch();
        assert_eq!(expected, response.into_string().unwrap());
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct SimpleReindeer {
    strength: u32,
}

#[post("/4/strength", data = "<reindeers>")]
fn reindeer_cheer(reindeers: Json<Vec<SimpleReindeer>>) -> String {
    reindeers
        .iter()
        .map(|r| r.strength)
        .sum::<u32>()
        .to_string()
}

#[cfg(test)]
#[test]
fn reindeer_cheer_test() {
    use rocket::http::ContentType;

    let client = test_client!();
    let response = client
        .post("/4/strength")
        .header(ContentType::JSON)
        .body(
            r#"[
    { "name": "Dasher", "strength": 5 },
    { "name": "Dancer", "strength": 6 },
    { "name": "Prancer", "strength": 4 },
    { "name": "Vixen", "strength": 7 }
  ]"#,
        )
        .dispatch();

    assert_eq!("22", response.into_string().unwrap());
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

#[cfg(test)]
#[test]
fn reindeer_candy_test() {
    use rocket::http::ContentType;

    let client = test_client!();
    let response = client
        .post("/4/contest")
        .header(ContentType::JSON)
        .body(
            r#"[
    {
      "name": "Dasher",
      "strength": 5,
      "speed": 50.4,
      "height": 80,
      "antler_width": 36,
      "snow_magic_power": 9001,
      "favorite_food": "hay",
      "cAnD13s_3ATeN-yesT3rdAy": 2
    },
    {
      "name": "Dancer",
      "strength": 6,
      "speed": 48.2,
      "height": 65,
      "antler_width": 37,
      "snow_magic_power": 4004,
      "favorite_food": "grass",
      "cAnD13s_3ATeN-yesT3rdAy": 5
    }
  ]"#,
        )
        .dispatch();

    assert_eq!(
        r#"{"fastest":"Speeding past the finish line with a strength of 5 is Dasher","tallest":"Dasher is standing tall with his 36 cm wide antlers","magician":"Dasher could blast you away with a snow magic power of 9001","consumer":"Dancer ate lots of candies, but also some grass"}"#,
        response.into_string().unwrap()
    );
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

#[cfg(test)]
#[test]
fn elf_count_test() {
    let client = test_client!();

    for (expected, data) in [
        (r#"{"elf":4,"elf on a shelf":0,"shelf with no elf on it":1}"#, "The mischievous elf peeked out from behind the toy workshop, and another elf joined in the festive dance. Look, there is also an elf on that shelf!"),
        (r#"{"elf":5,"elf on a shelf":1,"shelf with no elf on it":1}"#, "there is an elf on a shelf on an elf. there is also another shelf in Belfast.")
    ] {
        let response = client.post("/6").body(data).dispatch();

        assert_eq!(expected, response.into_string().unwrap());
    }
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

#[cfg(test)]
#[test]
fn cookie_recipe_test() {
    use rocket::http::Header;

    let client = test_client!();
    let response = client
        .get("/7/decode")
        .header(Header::new(
            "Cookie",
            "recipe=eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
        ))
        .dispatch();

    assert_eq!(
        r#"{"flour":100,"chocolate chips":20}"#,
        response.into_string().unwrap()
    );
}

type Ingredients = HashMap<String, u32>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Recipe {
    recipe: Ingredients,
    pantry: Ingredients,
}

impl Recipe {
    fn bake(mut self) -> AfterBake {
        let cookies = self
            .recipe
            .iter()
            .map(|(ing, r_amt)| self.pantry.get(ing).map_or(0, |p_amt| p_amt / r_amt))
            .min()
            .unwrap_or(0);

        for (ing, p_amt) in &mut self.pantry {
            if let Some(r_amt) = self.recipe.get(ing) {
                *p_amt -= r_amt * cookies;
            }
        }

        AfterBake {
            cookies,
            pantry: self.pantry,
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct AfterBake {
    cookies: u32,
    pantry: Ingredients,
}

#[get("/7/bake")]
fn bake_cookies(header: CookieHeader) -> Result<Json<AfterBake>, Error> {
    let recipe: Recipe = serde_json::from_str(&header.value).map_err(|_| Error {
        message: "Invalid JSON",
    })?;

    Ok(Json(recipe.bake()))
}

#[cfg(test)]
#[test]
fn bake_cookies_test() {
    use rocket::http::Header;

    let client = test_client!();

    for (expected, header) in [
        (vec![
            r#""cookies":4"#,
            r#""flour":5"#,
            r#""butter":2002"#,
            r#""baking powder":825"#,
            r#""chocolate chips":257"#,
            r#""sugar":307"#,
        ], "recipe=eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xhdGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyIjozODUsInN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBvd2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319"),
        (vec![
            r#""cookies":0"#,
            r#""cobblestone":64"#,
            r#""stick":4"#,
        ], "recipe=eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiOnsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==")
    ] {
        eprintln!("{header}");
        let response = client.get("/7/bake").header(Header::new("Cookie", header)).dispatch();

        let body = response.into_string().unwrap();

        for fragment in expected {
            assert!(
                body.contains(fragment),
                "Failed asserting that '{body}' contains '{fragment}'"
            );
        }
    }
}

async fn pokemon_weight_kg(id: i64) -> Result<f64, Error> {
    let client = RustemonClient::default();
    let pkm = pokemon::get_by_id(id, &client).await.map_err(|_| Error {
        message: "Something went wrong",
    })?;

    #[allow(clippy::cast_precision_loss)]
    Ok((pkm.weight as f64) / 10.0)
}

#[get("/8/weight/<id>")]
async fn pokemon_weight(id: i64) -> Result<String, Error> {
    Ok(pokemon_weight_kg(id).await?.to_string())
}

#[cfg(test)]
#[test]
fn pokemon_weight_test() {
    let client = test_client!();
    let response = client.get("/8/weight/25").dispatch();

    assert_eq!("6", response.into_string().unwrap());
}

#[get("/8/drop/<id>")]
async fn pokemon_drop(id: i64) -> Result<String, Error> {
    let mass = pokemon_weight_kg(id).await?;
    let height = 10.0;
    let gravitational_acceleration = 9.825;
    // Thanks ChatGPT for these formulas
    let velocity = (2.0f64 * height * gravitational_acceleration).sqrt();
    let momentum = velocity * mass;

    Ok(momentum.to_string())
}

#[cfg(test)]
#[test]
fn pokemon_drop_test() {
    let client = test_client!();
    let response = client.get("/8/drop/25").dispatch();

    assert_eq!("84.10707461325713", response.into_string().unwrap());
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount(
        "/",
        routes![
            index,
            fake_error,
            exclusive_cube,
            reindeer_cheer,
            reindeer_candy,
            elf_count,
            cookie_recipe,
            bake_cookies,
            pokemon_weight,
            pokemon_drop
        ],
    )
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    Ok(rocket().into())
}
