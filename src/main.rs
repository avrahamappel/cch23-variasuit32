#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]

use std::collections::HashMap;
use std::env;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Instant;

use chrono::{DateTime, Datelike, Utc, Weekday};
use image::io::Reader;
use image::DynamicImage;
use rocket::form::Form;
use rocket::fs::{relative, NamedFile, TempFile};
use rocket::serde::json::{serde_json, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::prelude::*;
use sqlx::PgPool;
use ulid::Ulid;
use uuid::Uuid;

mod common;
use common::Error;
mod day_0;
mod day_1;
mod day_4;
mod day_6;
mod day_7;
mod day_8;

#[cfg(test)]
macro_rules! test_client {
    () => {
        rocket::local::blocking::Client::tracked(
            rocket::build()
                .mount("/", routes())
                .manage(Timekeeper::new()),
        )
        .unwrap()
    };
}

#[get("/11/assets/<path..>")]
async fn assets(path: PathBuf) -> Option<NamedFile> {
    let path = Path::new(relative!("assets")).join(path);

    NamedFile::open(path).await.ok()
}

#[derive(FromForm)]
struct Image<'f> {
    image: TempFile<'f>,
}

#[post("/11/red_pixels", data = "<image>")]
async fn count_red_pixels(mut image: Form<Image<'_>>) -> Result<String, Error> {
    let name = image.image.name().unwrap_or("some-image.png");
    let path = env::temp_dir().join(name);
    image.image.persist_to(path).await?;

    let img = Reader::open(image.image.path().ok_or(Error {
        message: "Temp file had no path",
    })?)?
    .with_guessed_format()?
    .decode()?;

    macro_rules! count_pixels {
        ($rgb_image:ident) => {
            count_pixels!($rgb_image, saturating_add)
        };
        ($rgb_image:ident, $add:ident) => {{
            let red_pxl_count = $rgb_image
                .pixels()
                .filter(|p| {
                    let [red, green, blue] = p.0;
                    red > green.$add(blue)
                })
                .count();

            Ok(red_pxl_count.to_string())
        }};
    }

    match img {
        DynamicImage::ImageRgb8(rgb_image) => count_pixels!(rgb_image),
        DynamicImage::ImageRgb16(rgb_image) => count_pixels!(rgb_image),
        DynamicImage::ImageRgb32F(rgb_image) => count_pixels!(rgb_image, add),

        _ => Err(Error {
            message: "Image was not RGB",
        }),
    }
}

struct Timekeeper {
    store: RwLock<HashMap<String, Instant>>,
}

impl Timekeeper {
    fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn put(&self, string: String) -> Result<(), Error> {
        self.store.write()?.insert(string, Instant::now());
        Ok(())
    }

    fn get(&self, string: String) -> Option<u64> {
        self.store
            .read()
            .ok()?
            .get(&string)
            .map(|inst| inst.elapsed().as_secs())
    }
}

#[post("/12/save/<string>")]
fn store_string(timekeeper: &State<Timekeeper>, string: String) -> Result<(), Error> {
    timekeeper.put(string)?;
    Ok(())
}

#[get("/12/load/<string>")]
fn get_string(timekeeper: &State<Timekeeper>, string: String) -> Option<String> {
    timekeeper.get(string).map(|u| u.to_string())
}

#[post("/12/ulids", data = "<ulids>")]
fn ulid2uuid(ulids: Json<Vec<&str>>) -> Result<Json<Vec<String>>, Error> {
    let try_uuids: Result<Vec<_>, _> = ulids
        .iter()
        .map(|s| {
            Ulid::from_string(s).map(|ulid| {
                let bytes = ulid.to_bytes();
                Uuid::from_bytes(bytes).to_string()
            })
        })
        .rev()
        .collect();

    Ok(Json(try_uuids?))
}

#[cfg(test)]
#[test]
fn ulid2uuid_test() {
    let client = test_client!();
    let response = client
        .post("/12/ulids")
        .body(
            r#"["01BJQ0E1C3Z56ABCD0E11HYX4M","01BJQ0E1C3Z56ABCD0E11HYX5N","01BJQ0E1C3Z56ABCD0E11HYX6Q","01BJQ0E1C3Z56ABCD0E11HYX7R","01BJQ0E1C3Z56ABCD0E11HYX8P"]"#,
        )
        .dispatch();

    assert_eq!(
        r#"["015cae07-0583-f94c-a5b1-a070431f7516","015cae07-0583-f94c-a5b1-a070431f74f8","015cae07-0583-f94c-a5b1-a070431f74d7","015cae07-0583-f94c-a5b1-a070431f74b5","015cae07-0583-f94c-a5b1-a070431f7494"]"#,
        response.into_string().unwrap()
    );
}

#[derive(Serialize, Default)]
#[serde(crate = "rocket::serde")]
struct UlidsAnalysis {
    #[serde(rename = "christmas eve")]
    xmas_eve: usize,
    weekday: usize,
    #[serde(rename = "in the future")]
    future: usize,
    #[serde(rename = "LSB is 1")]
    lsb1: usize,
}

impl UlidsAnalysis {
    fn new(ulids: &[Ulid], weekday: Weekday) -> Self {
        ulids.iter().fold(Self::default(), |mut analysis, ulid| {
            let datetime: DateTime<Utc> = ulid.datetime().into();

            if datetime.month() == 12 && datetime.day() == 24 {
                analysis.xmas_eve += 1;
            }
            if datetime.weekday() == weekday {
                analysis.weekday += 1;
            }
            if datetime > Utc::now() {
                analysis.future += 1;
            }
            if (ulid.random()).trailing_ones() >= 1 {
                analysis.lsb1 += 1;
            }
            analysis
        })
    }
}

#[post("/12/ulids/<weekday>", data = "<ulids>")]
fn ulids_analyze(weekday: u8, ulids: Json<Vec<&str>>) -> Result<Json<UlidsAnalysis>, Error> {
    let weekday = Weekday::try_from(weekday)?;
    let ulids = ulids
        .iter()
        .map(|s| Ulid::from_string(s))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(UlidsAnalysis::new(&ulids, weekday)))
}

#[cfg(test)]
#[test]
fn ulids_analyze_test() {
    use serde_json::json;

    let client = test_client!();
    let response = client
        .post("/12/ulids/5")
        .body(
            json!([
                "00WEGGF0G0J5HEYXS3D7RWZGV8",
                "76EP4G39R8JD1N8AQNYDVJBRCF",
                "018CJ7KMG0051CDCS3B7BFJ3AK",
                "00Y986KPG0AMGB78RD45E9109K",
                "010451HTG0NYWMPWCEXG6AJ8F2",
                "01HH9SJEG0KY16H81S3N1BMXM4",
                "01HH9SJEG0P9M22Z9VGHH9C8CX",
                "017F8YY0G0NQA16HHC2QT5JD6X",
                "03QCPC7P003V1NND3B3QJW72QJ"
            ])
            .to_string(),
        )
        .dispatch();

    assert_eq!(
        r#"{"christmas eve":3,"weekday":1,"in the future":2,"LSB is 1":5}"#,
        response.into_string().unwrap()
    );
}

struct DB {
    pool: PgPool,
}

#[get("/13/sql")]
async fn sql(db: &State<DB>) -> Result<String, Error> {
    let res = sqlx::query("SELECT 20231213").fetch_one(&db.pool).await?;
    let int: i32 = res.get(0);
    Ok(int.to_string())
}

#[post("/13/reset")]
async fn reset_db(db: &State<DB>) -> Result<(), Error> {
    db.pool.execute(include_str!("../db/schema.sql")).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[post("/13/orders", data = "<orders>")]
async fn place_orders(db: &State<DB>, orders: Json<Vec<Order>>) -> Result<(), Error> {
    for order in &*orders {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(&order.gift_name)
        .bind(order.quantity)
        .execute(&db.pool)
        .await?;
    }
    Ok(())
}

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct OrderTotal {
    total: i64,
}

#[get("/13/orders/total")]
async fn orders_sum(db: &State<DB>) -> Result<Json<OrderTotal>, Error> {
    let res: OrderTotal = sqlx::query_as("SELECT SUM(quantity) AS total FROM orders")
        .fetch_one(&db.pool)
        .await?;

    Ok(Json(res))
}

#[derive(Serialize, FromRow, Default)]
#[serde(crate = "rocket::serde")]
struct OrdersPopular {
    popular: Option<String>,
}

#[get("/13/orders/popular")]
async fn orders_popular(db: &State<DB>) -> Result<Json<OrdersPopular>, Error> {
    let res: OrdersPopular = sqlx::query_as(
        "SELECT gift_name AS popular FROM (SELECT gift_name, SUM(quantity) AS total FROM orders GROUP BY gift_name) AS g ORDER BY total DESC LIMIT 1"
    )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_default();
    Ok(Json(res))
}

fn routes() -> Vec<Route> {
    routes![
        assets,
        count_red_pixels,
        store_string,
        get_string,
        ulid2uuid,
        ulids_analyze,
        // sql,
        // reset_db,
        // place_orders,
        // orders_sum,
        // orders_popular
    ]
}

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .mount("/", day_0::routes())
        .mount("/", day_1::routes())
        .mount("/", day_4::routes())
        .mount("/", day_6::routes())
        .mount("/", day_7::routes())
        .mount("/", day_8::routes())
        .mount("/", routes())
        .manage(Timekeeper::new())
        .manage(DB { pool });

    Ok(rocket.into())
}
