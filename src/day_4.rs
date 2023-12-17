use std::ops::Deref;

use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{post, routes, Route};

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

pub fn routes() -> Vec<Route> {
    routes![reindeer_cheer, reindeer_candy,]
}

#[cfg(test)]
mod tests {
    use rocket::http::ContentType;

    use crate::common::test_client;

    #[test]
    fn reindeer_cheer_test() {
        use rocket::http::ContentType;

        let client = test_client(super::routes());
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

    #[test]
    fn reindeer_candy_test() {
        let client = test_client(super::routes());
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
}
