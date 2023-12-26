use rocket::serde::json::{json, Json, Value};
use rocket::serde::Deserialize;
use rocket::{get, post, routes, Route, State};
use sqlx::{Executor, Row};

use crate::common::{Error, DB};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Region {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[post("/reset")]
async fn reset(db: &State<DB>) -> Result<(), Error> {
    db.pool.execute(include_str!("../db/schema_18.sql")).await?;
    Ok(())
}

#[post("/orders", data = "<orders>")]
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

#[post("/regions", data = "<regions>")]
async fn insert_regions(db: &State<DB>, regions: Json<Vec<Region>>) -> Result<(), Error> {
    for region in regions.into_inner() {
        sqlx::query("INSERT INTO regions (id, name) VALUES ($1, $2)")
            .bind(region.id)
            .bind(region.name)
            .execute(&db.pool)
            .await?;
    }
    Ok(())
}

#[get("/regions/total")]
async fn order_totals_per_region(db: &State<DB>) -> Result<Json<Value>, Error> {
    let totals: Vec<_> = sqlx::query(
        "
        SELECT
          r.name AS region,
          SUM(o.quantity) AS total
        FROM regions r
        JOIN orders o ON o.region_id = r.id
        GROUP BY r.name
        ORDER BY r.name ASC
    ",
    )
    .fetch_all(&db.pool)
    .await?
    .iter()
    .map(|row| {
        json!({
            "region": row.get::<String, _>("region"),
            "total": row.get::<i64, _>("total")
        })
    })
    .collect();

    Ok(Json(Value::Array(totals)))
}

pub fn routes() -> Vec<Route> {
    routes![reset, place_orders, insert_regions, order_totals_per_region]
}
